import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  PublicKey,
  Keypair,
  LAMPORTS_PER_SOL,
  SystemProgram,
} from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  getMint,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { randomBytes } from "crypto";
import { BN } from "@coral-xyz/anchor";
import {
  awaitComputationFinalization,
  getArciumEnv,
  getCompDefAccOffset,
  getCompDefAccAddress,
  getMXEAccAddress,
  getMempoolAccAddress,
  getExecutingPoolAccAddress,
  getComputationAccAddress,
  deserializeLE,
  x25519,
  getArciumProgramId,
  getClockAccAddress,
  RescueCipher,
} from "@arcium-hq/client";
import * as os from "os";
import { expect } from "chai";
import {
  setupUserEncryption,
  getMXEPublicKeyWithRetry,
  generateNonce,
} from "./helpers/encryption";
import {
  deriveOrderbookPDA,
  deriveOrderAccountPDA,
  deriveVaultStatePDA,
  deriveVaultAuthorityPDA,
  getOrderBookState,
  getOrderAccount,
  accountExists,
  airdrop,
  deriveVaultPDA,
  createATAAndMintTokens,
  deriveSignerAccountPDA,
  deriveArciumFeePoolAccountAddress,
} from "./helpers/accounts";
import {
  initSubmitOrderCompDef,
  initMatchOrdersCompDef,
  initInitOrderBookCompDef,
  readKpJson,
} from "./helpers/computation";

describe("Dark Pool Matching Engine - Core Functionality Tests", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.MatchingEngine as Program<MatchingEngine>;
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const arciumEnv = getArciumEnv();

  // Test accounts
  let authority: Keypair;
  let backendKeypair: Keypair;
  let user1: Keypair;
  let user2: Keypair;
  let baseMint: PublicKey;
  let quoteMint: PublicKey;
  let OrderbookPDA: PublicKey;
  let user1token1ATA: PublicKey;
  let user1token2ATA: PublicKey;
  let user2token1ATA: PublicKey;
  let user2token2ATA: PublicKey;

  // Event helper
  type Event = anchor.IdlEvents<typeof program.idl>;
  const awaitEvent = async <E extends keyof Event>(
    eventName: E,
    timeoutMs: number = 60000
  ): Promise<Event[E]> => {
    let listenerId: number;
    let timeoutId: NodeJS.Timeout;
    const event = await new Promise<Event[E]>((resolve, reject) => {
      listenerId = program.addEventListener(eventName, (event) => {
        if (timeoutId) clearTimeout(timeoutId);
        resolve(event);
      });
      timeoutId = setTimeout(() => {
        program.removeEventListener(listenerId);
        reject(new Error(`Event ${eventName} timed out after ${timeoutMs}ms`));
      }, timeoutMs);
    });
    await program.removeEventListener(listenerId);
    return event;
  };

  before(async () => {
    console.log("\n========================================");
    console.log("Setting up test environment...");
    console.log("========================================\n");

    // Load authority from default Solana config
    authority = readKpJson(`${os.homedir()}/.config/solana/id.json`);
    console.log("Authority:", authority.publicKey.toBase58());

    // Generate test accounts
    backendKeypair = Keypair.generate();
    user1 = Keypair.generate();
    user2 = Keypair.generate();

    console.log("Backend:", backendKeypair.publicKey.toBase58());
    console.log("User 1:", user1.publicKey.toBase58());
    console.log("User 2:", user2.publicKey.toBase58());

    await airdrop(provider, user1.publicKey, 100 * LAMPORTS_PER_SOL);
    await airdrop(provider, user2.publicKey, 100 * LAMPORTS_PER_SOL);

    // initialize a payer account to make token mints and their authority.
    const mintAuthority = Keypair.generate();
    await airdrop(provider, mintAuthority.publicKey, 100 * LAMPORTS_PER_SOL);

    // make two different tokens with same authority and then mint those tokens to both the users
    const token1Mint = await createMint(
      provider.connection,
      mintAuthority,
      mintAuthority.publicKey,
      null,
      9
    );
    const token2Mint = await createMint(
      provider.connection,
      mintAuthority,
      mintAuthority.publicKey,
      null,
      9
    );
    const ata1 = await createATAAndMintTokens(
      provider,
      user1.publicKey,
      token1Mint,
      mintAuthority,
      1000*LAMPORTS_PER_SOL
    );
    const ata2 = await createATAAndMintTokens(
      provider,
      user1.publicKey,
      token2Mint,
      mintAuthority,
      1000*LAMPORTS_PER_SOL
    );

    const ata3 = await createATAAndMintTokens(
      provider,
      user2.publicKey,
      token1Mint,
      mintAuthority,
      1000*LAMPORTS_PER_SOL
    );
    const ata4 = await createATAAndMintTokens(
      provider,
      user2.publicKey,
      token2Mint,
      mintAuthority,
      1000*LAMPORTS_PER_SOL
    );
    console.log("Minted tokens to users\n");

    // For now, use placeholder mints (in real test, create actual SPL tokens)
    baseMint = token1Mint;
    quoteMint = token2Mint;
    user1token1ATA = ata1;
    user1token2ATA = ata2;
    user2token1ATA = ata3;
    user2token2ATA = ata4;

    [OrderbookPDA] = deriveOrderbookPDA(program.programId);
    console.log("Orderbook PDA:", OrderbookPDA.toBase58());
  });

  describe("Suite 1.1: Program Initialization", () => {
    it("Test 1.1.1: Should initialize program with correct state", async () => {
      console.log("\n--- Test 1.1.1: Initialize Program ---");

      // Generate x25519 keypair for backend encryption (needed for verification)
      const backendSecretKey = x25519.utils.randomSecretKey();
      const backendPublicKey = x25519.getPublicKey(backendSecretKey);

      // Check if account already exists
      const accountAlreadyExists = await accountExists(provider, OrderbookPDA);
      console.log(
        "OrderBookState account already exists:",
        accountAlreadyExists
      );

      if (!accountAlreadyExists) {
        // Initialize program
        const tx = await program.methods
          .initialize(Array.from(backendPublicKey), baseMint, quoteMint)
          .accountsPartial({
            authority: authority.publicKey,
            orderBookState: OrderbookPDA,
            systemProgram: SystemProgram.programId,
          })
          .signers([authority])
          .rpc({ commitment: "confirmed" });

        console.log("Initialize tx:", tx);

        // Wait a moment for the account to be created
        await new Promise((resolve) => setTimeout(resolve, 1000));
      } else {
        console.log("Skipping initialization - account already exists");
      }

      // Fetch and verify account state
      const orderBookState = await getOrderBookState(program);
      console.log("OrderBookState fetched:", orderBookState);

      // Assertions
      expect(orderBookState).to.exist;
      expect(orderBookState.authority.toString()).to.equal(
        authority.publicKey.toString(),
        "Authority should match"
      );

      expect(orderBookState.orderbookNonce.toString()).to.equal(
        "0",
        "Initial orderbook nonce should be 0"
      );

      expect(Buffer.from(orderBookState.backendPubkey)).to.deep.equal(
        Buffer.from(backendPublicKey),
        "Backend pubkey should match"
      );

      expect(orderBookState.baseMint.toString()).to.equal(
        baseMint.toString(),
        "Base mint should match"
      );

      expect(orderBookState.quoteMint.toString()).to.equal(
        quoteMint.toString(),
        "Quote mint should match"
      );

      expect(orderBookState.totalOrdersProcessed.toString()).to.equal(
        "0",
        "Total orders should be 0"
      );

      expect(orderBookState.totalMatches.toString()).to.equal(
        "0",
        "Total matches should be 0"
      );

      // Verify orderbook data is initialized (all zeros)
      const allZeros = orderBookState.orderbookData.every(
        (byte: number) => byte === 0
      );
      expect(allZeros).to.be.true;

      console.log("✓ Program initialized successfully");
      console.log("  - Authority:", orderBookState.authority.toBase58());
      // console.log("  - Orderbook nonce:", orderBookState.orderBookNonce.toString());
      console.log("  - Base mint:", orderBookState.baseMint.toBase58());
      console.log("  - Quote mint:", orderBookState.quoteMint.toBase58());
    });

    it("Test 1.1.2: Should initialize computation definitions", async () => {
      console.log("\n--- Test 1.1.2: Initialize Computation Definitions ---");

      console.log("Initializing submit_order computation definition...");
      let submitOrderCompDefSig;
      try {
        submitOrderCompDefSig = await initSubmitOrderCompDef(
          program,
          authority,
          false,
          false
        );
        console.log("Submit order comp def sig:", submitOrderCompDefSig);
      } catch (error) {
        if (error.message.includes("already in use")) {
          console.log("Submit order comp def already exists, skipping...");
          submitOrderCompDefSig = "already_exists";
        } else {
          throw error;
        }
      }
      expect(submitOrderCompDefSig).to.exist;

      console.log("\nInitializing match_orders computation definition...");
      let matchOrdersCompDefSig;
      try {
        matchOrdersCompDefSig = await initMatchOrdersCompDef(
          program,
          authority,
          false,
          false
        );
        console.log("Match orders comp def sig:", matchOrdersCompDefSig);
      } catch (error) {
        if (error.message.includes("already in use")) {
          console.log("Match orders comp def already exists, skipping...");
          matchOrdersCompDefSig = "already_exists";
        } else {
          throw error;
        }
      }
      expect(matchOrdersCompDefSig).to.exist;

      // Verify comp defs are accessible
      const submitOrderCompDefPDA = getCompDefAccAddress(
        program.programId,
        Buffer.from(getCompDefAccOffset("submit_order")).readUInt32LE()
      );

      const matchOrdersCompDefPDA = getCompDefAccAddress(
        program.programId,
        Buffer.from(getCompDefAccOffset("match_orders")).readUInt32LE()
      );

      // Fetch accounts to verify they exist
      const submitOrderCompDef = await provider.connection.getAccountInfo(
        submitOrderCompDefPDA
      );
      expect(submitOrderCompDef).to.exist;

      const matchOrdersCompDef = await provider.connection.getAccountInfo(
        matchOrdersCompDefPDA
      );
      expect(matchOrdersCompDef).to.exist;

      console.log("✓ Computation definitions initialized successfully");
      console.log(
        "  - submit_order comp def PDA:",
        submitOrderCompDefPDA.toBase58()
      );
      console.log(
        "  - match_orders comp def PDA:",
        matchOrdersCompDefPDA.toBase58()
      );
    });

    it("Test 1.1.3: Should retrieve MXE public key", async () => {
      console.log("\n--- Test 1.1.3: Retrieve MXE Public Key ---");

      const mxePublicKey = await getMXEPublicKeyWithRetry(
        provider,
        program.programId
      );

      expect(mxePublicKey).to.exist;
      expect(mxePublicKey.length).to.equal(
        32,
        "MXE public key should be 32 bytes"
      );

      console.log("✓ MXE public key retrieved successfully");
      console.log(
        "  - MXE pubkey (hex):",
        Buffer.from(mxePublicKey).toString("hex")
      );

      // Test key exchange
      const userPrivateKey = x25519.utils.randomSecretKey();
      const sharedSecret = x25519.getSharedSecret(userPrivateKey, mxePublicKey);

      expect(sharedSecret).to.exist;
      expect(sharedSecret.length).to.equal(
        32,
        "Shared secret should be 32 bytes"
      );

      console.log("✓ Key exchange works correctly");
    });
  });

  describe("Suite 1.2: Vault Management", () => {
    it("Test 1.2.1: Should initialize user vault (base + quote)", async () => {
      console.log("\n--- Test 1.2.1: Initialize User Vaults ---");

      // TODO: Implement vault initialization
      // This requires the initialize_vault instruction to be implemented
      console.log("⚠ Vault initialization test - To be implemented");
      console.log("  Requires: initialize_vault instruction");
      console.log("  Creates: VaultState PDAs for base and quote tokens");
    });

    it("Test 1.2.2: Should deposit tokens to vault", async () => {
      console.log("\n--- Test 1.2.2: Deposit to Vault ---");

      // TODO: Implement deposit test
      console.log("⚠ Deposit test - To be implemented");
      console.log("  Requires: deposit_to_vault instruction");
    });

    it("Test 1.2.3: Should track vault state correctly", async () => {
      console.log("\n--- Test 1.2.3: Track Vault State ---");

      // TODO: Implement vault state tracking test
      console.log("⚠ Vault state tracking test - To be implemented");
    });

    it("Test 1.2.4: Should withdraw from vault", async () => {
      console.log("\n--- Test 1.2.4: Withdraw from Vault ---");

      // TODO: Implement withdrawal test
      console.log("⚠ Withdrawal test - To be implemented");
      console.log("  Requires: withdraw_from_vault instruction");
    });
  });

  describe("Suite 1.3: Order Submission", () => {
    it("Should submit buy order", async () => {



      let submitOrderCompDefSig;
      try {
        submitOrderCompDefSig = await initSubmitOrderCompDef(
          program,
          authority,
          false,
          false
        );
        console.log("Submit order comp def sig:", submitOrderCompDefSig);
      } catch (error) {
        if (error.message.includes("already in use")) {
          console.log("Submit order comp def already exists, skipping...");
          submitOrderCompDefSig = "already_exists";
        } else {
          throw error;
        }
      }
      expect(submitOrderCompDefSig).to.exist;


      let matchOrdersCompDefSig;
      try {
        matchOrdersCompDefSig = await initMatchOrdersCompDef(
          program,
          authority,
          false,
          false
        );
        console.log("Match orders comp def sig:", matchOrdersCompDefSig);
      } catch (error) {
        if (error.message.includes("already in use")) {
          console.log("Match orders comp def already exists, skipping...");
          matchOrdersCompDefSig = "already_exists";
        } else {
          throw error;
        }
      }
      expect(matchOrdersCompDefSig).to.exist;

      let initOrderBookCompDefSig;
      try {
        initOrderBookCompDefSig = await initInitOrderBookCompDef(
          program,
          authority,
          false,
          false
        );
      } catch (error) {
        if (error.message.includes("already in use")) {
          console.log("Init order book comp def already exists, skipping...");
          initOrderBookCompDefSig = "already_exists";
        } else {
          throw error;
        }
      }
      expect(initOrderBookCompDefSig).to.exist;

      // 1. Setup encryption
      const { publicKey, cipher } = await setupUserEncryption(
        provider,
        program.programId
      );

      const [vaultPDA] = deriveVaultPDA(
        baseMint,
        user1.publicKey,
        program.programId
      );
      const [vaultStatePDA] = deriveVaultStatePDA(
        baseMint,
        user1.publicKey,
        program.programId
      );
      const [vaultAuthorityPDA] = deriveVaultAuthorityPDA(program.programId);
      // first initialize the vault
      await program.methods
        .initializeVault()
        .accountsPartial({
          user: user1.publicKey,
          mint: baseMint,
          vault: vaultPDA,
          vaultState: vaultStatePDA,
          vaultAuthority: vaultAuthorityPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc({ commitment: "confirmed" });

      console.log("Vault initialized");

      console.log("user1", user1.publicKey.toBase58());
      console.log("baseMint", baseMint.toBase58());
      console.log("user1token1ATA", user1token1ATA.toBase58());

      // then deposit to the vault
      await program.methods
        .depositToVault(new BN(100))
        .accountsPartial({
          user: user1.publicKey,
          userTokenAccount: user1token1ATA,
          vault: vaultPDA,
        })
        .signers([user1])
        .rpc({ commitment: "confirmed" });

      console.log("Tokens deposited to vault");

      // 2. Prepare order (using smaller values to reduce stack usage)
      const amount =10;
      const price = 5;
      const submitOrderComputationOffset = new anchor.BN(randomBytes(8), "hex");;

      // 3. Read initial nonce
      const before = await getOrderBookState(program);
      const initialNonce = before.orderBookNonce;

      // 4. Listen for event
      const eventPromise = awaitEvent("orderProcessedEvent");

      const orderId = new BN(12);

      const [orderAccountPDA] = deriveOrderAccountPDA(
        orderId,
        user1.publicKey,
        program.programId
      );

      console.log("=== submitOrder Accounts ===");
      console.log("User:", user1.publicKey.toBase58());
      console.log("Vault PDA:", vaultPDA.toBase58());
      console.log("Vault State PDA:", vaultStatePDA.toBase58());
      console.log("Order Account PDA:", orderAccountPDA.toBase58());
      console.log("Orderbook PDA:", OrderbookPDA.toBase58());
      console.log("program id", program.programId.toBase58());



      // verify if arcium accounts are correct
      console.log("arcium fee pool account", deriveArciumFeePoolAccountAddress().toBase58());
      console.log("arcium clock account", getClockAccAddress().toBase58());
      console.log("arcium program id", getArciumProgramId().toBase58());
      console.log("arcium cluster pubkey", arciumEnv.arciumClusterPubkey.toBase58());
      console.log("arcium mxe account", getMXEAccAddress(program.programId).toBase58());
      console.log("arcium mempool account", getMempoolAccAddress(program.programId).toBase58());
      console.log("arcium executing pool account", getExecutingPoolAccAddress(program.programId).toBase58());
      console.log("arcium comp def account", getCompDefAccAddress(program.programId, Buffer.from(getCompDefAccOffset("submit_order")).readUInt32LE()).toBase58());
      console.log("arcium computation account", getComputationAccAddress(program.programId, submitOrderComputationOffset).toBase58());

      // Get MXE public key
      const mxePublicKey = await getMXEPublicKeyWithRetry(
        provider as anchor.AnchorProvider,
        program.programId
      );

      // Generate encryption keys for User1
      const User1PrivateKey = x25519.utils.randomSecretKey();
      const User1PublicKey = x25519.getPublicKey(User1PrivateKey);
      const User1SharedSecret = x25519.getSharedSecret(
        User1PrivateKey,
        mxePublicKey
      );
      const User1Cipher = new RescueCipher(User1SharedSecret);

      const User1Nonce = randomBytes(16);
      const User1Ciphertext = User1Cipher.encrypt(
        [BigInt(amount), BigInt(price)],
        User1Nonce
      );
  
      // 5. Submit order
      await program.methods
        .submitOrder(
          Array.from(User1Ciphertext[0]),
          Array.from(User1Ciphertext[1]),
          Array.from(User1PublicKey),
          0, // buy
          submitOrderComputationOffset,
          orderId,
          new anchor.BN(deserializeLE(User1Nonce).toString())
        )
        .accountsPartial({
          computationAccount: getComputationAccAddress(
            program.programId,
            submitOrderComputationOffset
          ),
          user: user1.publicKey,
          signPdaAccount: deriveSignerAccountPDA(program.programId),
          poolAccount: deriveArciumFeePoolAccountAddress(),
          clusterAccount: arciumEnv.arciumClusterPubkey,
          mxeAccount: getMXEAccAddress(program.programId),
          mempoolAccount: getMempoolAccAddress(program.programId),
          executingPool: getExecutingPoolAccAddress(program.programId),
          compDefAccount: getCompDefAccAddress(
            program.programId,
            Buffer.from(
              getCompDefAccOffset("submit_order")
            ).readUInt32LE()
          ),
          clockAccount: getClockAccAddress(),
          systemProgram: SystemProgram.programId, 
          arciumProgram: getArciumProgramId(),        
          baseMint: baseMint,
          vault: vaultPDA,
          orderAccount: orderAccountPDA,
          vaultState: vaultStatePDA,
          orderbookState: OrderbookPDA,
        })
        .signers([user1])
        .rpc({ commitment: "confirmed" });

      console.log("meow meow meow meow")

      // 6. Wait for MPC finalization
      await awaitComputationFinalization(
        provider,
        submitOrderComputationOffset,
        program.programId,
        "confirmed"
      );

      console.log("=============== Order submitted successfully ===============");

      // 7. Get event
      const event = await eventPromise;
      expect(event.success).to.be.true;

      // 8. CRITICAL: Verify nonce incremented
      const after = await getOrderBookState(program);
      expect(after.orderBookNonce.toString()).to.equal(
        initialNonce.add(new BN(1)).toString()
      );

      // 9. Verify OrderAccount created
      const orderAccount = await getOrderAccount(
        program,
        new BN(event.orderId),
        user1.publicKey
      );
      expect(orderAccount.status).to.equal(1); // Processing
    });

    it("Test 1.3.3: Should handle user pubkey chunking correctly", async () => {
      console.log("\n--- Test 1.3.3: User Pubkey Chunking ---");

      // Test pubkey chunking
      const testPubkey = user1.publicKey.toBuffer();

      // Split into 4x u64 chunks
      const chunks = [
        BigInt("0x" + testPubkey.slice(0, 8).toString("hex")),
        BigInt("0x" + testPubkey.slice(8, 16).toString("hex")),
        BigInt("0x" + testPubkey.slice(16, 24).toString("hex")),
        BigInt("0x" + testPubkey.slice(24, 32).toString("hex")),
      ];

      console.log("Original pubkey:", user1.publicKey.toBase58());
      console.log(
        "Chunks:",
        chunks.map((c) => c.toString(16))
      );

      // Reconstruct
      const reconstructed = Buffer.concat([
        Buffer.from(chunks[0].toString(16).padStart(16, "0"), "hex"),
        Buffer.from(chunks[1].toString(16).padStart(16, "0"), "hex"),
        Buffer.from(chunks[2].toString(16).padStart(16, "0"), "hex"),
        Buffer.from(chunks[3].toString(16).padStart(16, "0"), "hex"),
      ]);

      console.log(
        "Reconstructed pubkey:",
        new PublicKey(reconstructed).toBase58()
      );

      console.log("✓ Pubkey chunking works correctly");
    });

    it("Test 1.3.4: Should encrypt/decrypt correctly", async () => {
      console.log("\n--- Test 1.3.4: Encryption/Decryption ---");

      const { cipher } = await setupUserEncryption(provider, program.programId);

      const amount = BigInt(100);
      const price = BigInt(50);
      const plaintext = [amount, price];
      const nonce = generateNonce();

      console.log("Original values:");
      console.log("  - Amount:", amount.toString());
      console.log("  - Price:", price.toString());

      // Encrypt
      const ciphertext = cipher.encrypt(plaintext, nonce);
      console.log(
        "Encrypted:",
        ciphertext.map(
          (c) => Buffer.from(c).toString("hex").slice(0, 16) + "..."
        )
      );

      // Decrypt
      const decrypted = cipher.decrypt(ciphertext, nonce);
      console.log("Decrypted:");
      console.log("  - Amount:", decrypted[0].toString());
      console.log("  - Price:", decrypted[1].toString());

      expect(decrypted[0]).to.equal(amount);
      expect(decrypted[1]).to.equal(price);

      console.log("✓ Encryption/decryption works correctly");
    });
  });

  describe("Suite 1.4: Order Matching", () => {
    it("Test 1.4.1: Should trigger matching with valid orders", async () => {
      console.log("\n--- Test 1.4.1: Trigger Matching ---");
      console.log("⚠ Matching test - To be implemented");
      console.log("  Requires:");
      console.log("  - Submit buy and sell orders");
      console.log("  - Trigger matching computation");
      console.log("  - Verify nonce increment (CRITICAL!)");
      console.log("  - Verify MatchResultEvent");
    });

    it("Test 1.4.2: Should enforce rate limiting (15s)", async () => {
      console.log("\n--- Test 1.4.2: Rate Limiting ---");
      console.log("⚠ Rate limiting test - To be implemented");
    });
  });

  describe("Suite 1.5: Backend Decryption", () => {
    it("Test 1.5.1: Should decrypt match results", async () => {
      console.log("\n--- Test 1.5.1: Backend Decryption ---");
      console.log("⚠ Decryption test - To be implemented");
      console.log("  Requires:");
      console.log("  - Setup backend encryption keys");
      console.log("  - Listen for MatchResultEvent");
      console.log("  - Decrypt match ciphertext with match nonce");
    });
  });

  describe("Suite 1.6: Settlement", () => {
    it("Test 1.6.1: Should derive vault PDAs correctly", async () => {
      console.log("\n--- Test 1.6.1: Derive Vault PDAs ---");

      const [baseVaultPDA] = deriveVaultStatePDA(
        baseMint,
        user1.publicKey,
        program.programId
      );

      const [quoteVaultPDA] = deriveVaultStatePDA(
        quoteMint,
        user1.publicKey,
        program.programId
      );

      console.log("User 1 vaults:");
      console.log("  - Base vault PDA:", baseVaultPDA.toBase58());
      console.log("  - Quote vault PDA:", quoteVaultPDA.toBase58());

      console.log("✓ Vault PDA derivation works correctly");
    });

    it("Test 1.6.2: Should execute settlement", async () => {
      console.log("\n--- Test 1.6.2: Execute Settlement ---");
      console.log("⚠ Settlement test - To be implemented");
      console.log("  Requires:");
      console.log("  - Match results from previous test");
      console.log("  - Call execute_settlement instruction");
      console.log("  - Verify token transfers");
    });
  });
});
