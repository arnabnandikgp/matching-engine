import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { randomBytes } from "crypto";
import {
  RescueCipher,
  getMXEPublicKey,
  x25519,
} from "@arcium-hq/client";

export interface EncryptionSetup {
  privateKey: Uint8Array;
  publicKey: Uint8Array;
  cipher: RescueCipher;
  mxePublicKey: Uint8Array;
}

/**
 * Setup user encryption with x25519 key exchange and RescueCipher
 */
export async function setupUserEncryption(
  provider: anchor.AnchorProvider,
  programId: PublicKey
): Promise<EncryptionSetup> {
  const privateKey = x25519.utils.randomSecretKey();
  const publicKey = x25519.getPublicKey(privateKey);

  const mxePublicKey = await getMXEPublicKeyWithRetry(provider, programId);
  const sharedSecret = x25519.getSharedSecret(privateKey, mxePublicKey);
  const cipher = new RescueCipher(sharedSecret);

  return {
    privateKey,
    publicKey,
    cipher,
    mxePublicKey,
  };
}

/**
 * Setup backend encryption (for decrypting match results)
 */
export async function setupBackendEncryption(
  provider: anchor.AnchorProvider,
  programId: PublicKey,
  backendPrivateKey: Uint8Array
): Promise<EncryptionSetup> {
  const backendPublicKey = x25519.getPublicKey(backendPrivateKey);
  const mxePublicKey = await getMXEPublicKeyWithRetry(provider, programId);
  const sharedSecret = x25519.getSharedSecret(backendPrivateKey, mxePublicKey);
  const cipher = new RescueCipher(sharedSecret);

  return {
    privateKey: backendPrivateKey,
    publicKey: backendPublicKey,
    cipher,
    mxePublicKey,
  };
}

/**
 * Get MXE public key with retry logic
 */
export async function getMXEPublicKeyWithRetry(
  provider: anchor.AnchorProvider,
  programId: PublicKey,
  maxRetries: number = 10,
  retryDelayMs: number = 500
): Promise<Uint8Array> {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      const mxePublicKey = await getMXEPublicKey(provider, programId);
      if (mxePublicKey) {
        return mxePublicKey;
      }
    } catch (error) {
      console.log(`Attempt ${attempt} failed to fetch MXE public key:`, error);
    }

    if (attempt < maxRetries) {
      console.log(
        `Retrying in ${retryDelayMs}ms... (attempt ${attempt}/${maxRetries})`
      );
      await new Promise((resolve) => setTimeout(resolve, retryDelayMs));
    }
  }

  throw new Error(
    `Failed to fetch MXE public key after ${maxRetries} attempts`
  );
}

/**
 * Generate random nonce for encryption
 */
export function generateNonce(): Uint8Array {
  return randomBytes(16);
}

/**
 * Simple encrypt function that returns raw result from RescueCipher
 * Use this when passing to Arcium instructions
 */
export function encrypt(
  cipher: RescueCipher,
  plaintext: bigint[],
  nonce: Uint8Array
): any[] {
  return cipher.encrypt(plaintext, nonce);
}

/**
 * Simple decrypt function
 */
export function decrypt(
  cipher: RescueCipher,
  ciphertext: any[],
  nonce: Uint8Array | number[]
): bigint[] {
  const nonceBytes = nonce instanceof Uint8Array ? nonce : Uint8Array.from(nonce);
  return cipher.decrypt(ciphertext, nonceBytes);
}

/**
 * Encrypt order data (amount and price)
 * Returns array of Uint8Arrays, where each element is a 32-byte encrypted field
 */
export function encryptOrderData(
  cipher: RescueCipher,
  amount: bigint,
  price: bigint,
  nonce: Uint8Array
): any[] {
  const plaintext = [amount, price];
  // cipher.encrypt returns number[][] internally, but we treat it as Uint8Array[]
  const encrypted = cipher.encrypt(plaintext, nonce);
  // Convert to Uint8Array if needed for compatibility
  return encrypted.map((arr: any) => 
    arr instanceof Uint8Array ? arr : Uint8Array.from(arr)
  );
}

/**
 * Decrypt match results
 * Takes array of encrypted fields and returns decrypted bigints
 */
export function decryptMatchResults(
  cipher: RescueCipher,
  ciphertext: any[],
  nonce: Uint8Array | number[]
): bigint[] {
  // Convert nonce to Uint8Array if it's a number array
  const nonceBytes = nonce instanceof Uint8Array ? nonce : Uint8Array.from(nonce);
  return cipher.decrypt(ciphertext, nonceBytes);
}
