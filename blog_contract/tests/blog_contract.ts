import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BlogContract } from "../target/types/blog_contract";

import { BN } from "bn.js";
import { assert } from "chai";

type BNType = InstanceType<typeof BN>;

describe("blog_contract", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.blogContract as Program<BlogContract>;
  const payer = provider.wallet;


  // it("Creates a user blog account", async () => {
  //   await program.methods
  //     .createUserBlogAccount()
  //     .accounts({
  //       payer: payer.publicKey,
  //     })
  //     .rpc();
    
  //   // Fetch the user account PDA
  //   const [userPda] = anchor.web3.PublicKey.findProgramAddressSync(
  //     [Buffer.from("user"), payer.publicKey.toBuffer()],
  //     program.programId
  //   );

  //   const user = await program.account.user.fetch(userPda);

  //   assert.ok(user.postCount.toNumber() === 0, "Post count should start at 0");
  //   console.log("✅ User blog account created:", userPda.toBase58());
  // });

  // it("Creates a post", async () => {
  //   const title = "My first post";
  //   const content = "Hello, this is on-chain content!";

  //   const tx = await program.methods
  //     .createPost(title, content)
  //     .accounts({
  //       payer: payer.publicKey,
  //     })
  //     .rpc();

  //     // Fetch user again to check postCount increment
  //   const [userPda] = anchor.web3.PublicKey.findProgramAddressSync(
  //     [Buffer.from("user"), payer.publicKey.toBuffer()],
  //     program.programId
  //   );
  //   const user = await program.account.user.fetch(userPda);
  //   console.log(user);
  //   assert.ok(user.postCount.toNumber() === 1, "Post count should be 1 after first post");

  //   const index = new BN(0);
  //   const [postPda] = anchor.web3.PublicKey.findProgramAddressSync(
  //     [Buffer.from("post"), payer.publicKey.toBuffer(), index.toArrayLike(Buffer, "le", 8)],
  //     program.programId
  //   );
  //   const post = await program.account.post.fetch(postPda);

  //   assert.equal(post.title, title, "Post title should match");
  //   assert.equal(post.content, content, "Post content should match");
  //   assert.ok(post.likes.toNumber() === 0, "New post should start with 0 likes");

  //   console.log("✅ Post created:", postPda.toBase58());;
  // });

  // it("User Likes a Post", async () => {

  //   let postIndex = new BN(0); 
  //   // Derive the user PDA
  //   let [userPda] = anchor.web3.PublicKey.findProgramAddressSync(
  //     [Buffer.from("user"), payer.publicKey.toBuffer()],
  //     program.programId
  //   );

  //   let [postPda] = anchor.web3.PublicKey.findProgramAddressSync(
  //     [Buffer.from("post"), payer.publicKey.toBuffer(), postIndex.toArrayLike(Buffer, "le", 8)],
  //     program.programId
  //   );
  //   const tx = await program.methods
  //     .likePost(payer.publicKey, postIndex)
  //     .accounts({
  //       payer: payer.publicKey,
  //     })
  //     .rpc();

  //   // Fetch the post again to verify likes incremented
  // const post = await program.account.post.fetch(postPda);

  // assert.ok(post.likes.toNumber() === 1, "Post should have 1 like after first like");
  // console.log("✅ Post liked successfully. Total likes:", post.likes.toNumber());


  // });

  it("Simulation: 20 users, 20 posts each, and random likes", async () => {
    const NUM_USERS = 20;
    const POSTS_PER_USER = 20;

    // Step 1: Generate user keypairs
    const users: anchor.web3.Keypair[] = [];
    for (let i = 0; i < NUM_USERS; i++) {
      const user = anchor.web3.Keypair.generate();
      users.push(user);

      // Airdrop some SOL to each user for tx fees
      const sig = await provider.connection.requestAirdrop(user.publicKey, 2e9);
      const latestBlockhash = await provider.connection.getLatestBlockhash();

      await provider.connection.confirmTransaction({
        signature: sig,
        blockhash: latestBlockhash.blockhash,
        lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
      });
      

      // Create user blog account
      await program.methods
        .createUserBlogAccount()
        .accounts({
          payer: user.publicKey,
        })
        .signers([user])
        .rpc();
    }

    // Step 2: Each user creates POSTS_PER_USER posts
    for (let i = 0; i < NUM_USERS; i++) {
      const user = users[i];
      for (let j = 0; j < POSTS_PER_USER; j++) {
        const title = `User ${i} Post ${j}`;
        const content = `This is post ${j} from user ${i}`;

        await program.methods
          .createPost(title, content)
          .accounts({
            payer: user.publicKey,
          })
          .signers([user])
          .rpc();
      }

      // Verify postCount increments properly
      const [userPda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("user"), user.publicKey.toBuffer()],
        program.programId
      );
      const userAccount = await program.account.user.fetch(userPda);
      assert.ok(
        userAccount.postCount.toNumber() === POSTS_PER_USER,
        `User ${i} should have ${POSTS_PER_USER} posts`
      );
    }

    // Step 3: Random likes (each user likes 5 random posts not their own)
    const totalPosts: { postPda: anchor.web3.PublicKey; owner: anchor.web3.Keypair; index: BNType }[] = [];


    for (let i = 0; i < NUM_USERS; i++) {
      for (let j = 0; j < POSTS_PER_USER; j++) {
        const index = new BN(j);
        const [postPda] = anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("post"), users[i].publicKey.toBuffer(), index.toArrayLike(Buffer, "le", 8)],
          program.programId
        );
        totalPosts.push({ postPda, owner: users[i], index });
      }
    }

    for (let i = 0; i < NUM_USERS; i++) {
      const liker = users[i];
      const randomPosts = totalPosts
        .filter((p) => !p.owner.publicKey.equals(liker.publicKey)) // don’t like own post
        .sort(() => 0.5 - Math.random())
        .slice(0, 5); // pick 5 random posts

      for (const p of randomPosts) {
        await program.methods
          .likePost(p.owner.publicKey, p.index)
          .accounts({
            payer: liker.publicKey,
          })
          .signers([liker])
          .rpc();
      }
    }

    // Step 4: Verify likes updated (at least some should be > 0)
    const likedPosts = await Promise.all(
      totalPosts.map(async (p) => {
        const post = await program.account.post.fetch(p.postPda);
        return post.likes.toNumber();
      })
    );

    const totalLikes = likedPosts.reduce((a, b) => a + b, 0);
    console.log("✅ Simulation complete. Total likes across posts:", totalLikes);
    assert.ok(totalLikes > 0, "There should be some likes in the simulation");


  });

});
