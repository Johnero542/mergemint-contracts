# Contributor FAQ

Answers to common questions about claiming and completing bounties on MergeMint.

---

**How do I find open bounties?**

Open bounties are listed on the MergeMint platform and indexed from on-chain events. Each bounty shows the title, description, reward amount, and reward token before you commit to anything.

---

**How do I claim a bounty?**

Call `claim_bounty` with your wallet address and the bounty ID. This assigns the bounty to you and moves its status to `in_progress`. Only one contributor can claim a given bounty — first claim wins.

---

**What happens if I claim a bounty but cannot complete it?**

Currently, nothing automatic happens. The bounty stays assigned to you and no one else can claim it. If you cannot complete the work, communicate with the bounty creator or verifier so they can make arrangements. A future version of the contract will introduce claim expiry to handle abandoned bounties automatically.

---

**Who is the verifier and how are they chosen?**

The verifier is the address that calls `complete_bounty` to release the reward. In practice this is typically the bounty creator or a trusted maintainer of the project. How verifiers are designated is determined off-chain by the project — the contract itself does not enforce a specific verifier address.

---

**What if the verifier never calls `complete_bounty`?**

At present, there is no on-chain timeout or escalation mechanism. If a verifier is unresponsive after you have completed the work, your recourse is off-chain: contact the project maintainers or raise the issue publicly. Automatic expiry and dispute mechanisms are planned for a future contract version.

---

**How is my reputation calculated?**

Each time a verifier calls `complete_bounty` for a bounty you completed, your reputation score increases by 10. It never decreases. Your profile also tracks total tokens earned and total bounties completed.

---

**How do I dispute a completion decision?**

There is currently no on-chain dispute mechanism. If you believe a completion was handled incorrectly — for example, a reward was not paid after work was accepted — raise the issue with the project maintainers. On-chain dispute resolution is a planned future feature.
