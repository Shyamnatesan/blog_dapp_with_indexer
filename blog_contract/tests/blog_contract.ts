import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BlogContract } from "../target/types/blog_contract";

import { BN } from "bn.js";
import { assert } from "chai";

describe("blog_contract", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.blogContract as Program<BlogContract>;
  const payer = provider.wallet;


  it("Creates a user blog account", async () => {
    await program.methods
      .createUserBlogAccount()
      .accounts({
        payer: payer.publicKey,
      })
      .rpc();
    
    // Fetch the user account PDA
    const [userPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user"), payer.publicKey.toBuffer()],
      program.programId
    );

    const user = await program.account.user.fetch(userPda);

    assert.ok(user.postCount.toNumber() === 0, "Post count should start at 0");
    console.log("✅ User blog account created:", userPda.toBase58());
  });

  it("Creates a post", async () => {
    const title = "My first post";
    const content = "Hello, this is on-chain content!";

    const tx = await program.methods
      .createPost(title, content)
      .accounts({
        payer: payer.publicKey,
      })
      .rpc();

      // Fetch user again to check postCount increment
    const [userPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user"), payer.publicKey.toBuffer()],
      program.programId
    );
    const user = await program.account.user.fetch(userPda);
    console.log(user);
    assert.ok(user.postCount.toNumber() === 1, "Post count should be 1 after first post");

    const index = new BN(0);
    const [postPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("post"), payer.publicKey.toBuffer(), index.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    const post = await program.account.post.fetch(postPda);

    assert.equal(post.title, title, "Post title should match");
    assert.equal(post.content, content, "Post content should match");
    assert.ok(post.likes.toNumber() === 0, "New post should start with 0 likes");

    console.log("✅ Post created:", postPda.toBase58());;
  });

  it("User Likes a Post", async () => {

    let postIndex = new BN(0); 
    // Derive the user PDA
    let [userPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user"), payer.publicKey.toBuffer()],
      program.programId
    );

    let [postPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("post"), payer.publicKey.toBuffer(), postIndex.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    const tx = await program.methods
      .likePost(payer.publicKey, postIndex)
      .accounts({
        payer: payer.publicKey,
      })
      .rpc();

    // Fetch the post again to verify likes incremented
  const post = await program.account.post.fetch(postPda);

  assert.ok(post.likes.toNumber() === 1, "Post should have 1 like after first like");
  console.log("✅ Post liked successfully. Total likes:", post.likes.toNumber());


  });

});
