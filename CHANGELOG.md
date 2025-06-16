## 2.0.0

- Added refund handler
- Added Cursor AI config
- Move to `@solana/kit, Codama and Kite for making TS clients
  - `create-codama-client.ts` is used to produce a TypeScript client from the Anchor IDL.
  - `program.methods.doThing()` becomes `programClient.getDoThingInstruction()`

## 2025

- Third party package managers and test tools are removed: the project now uses `npm` for installs and `node` for unit tests.
- Workaround for [web3.js punycode warning](https://stackoverflow.com/questions/68774489/punycode-is-deprecated-in-npm-what-should-i-replace-it-with) was applied
- Workaround for [Rust tools warnings](https://solana.stackexchange.com/questions/17777/unexpected-cfg-condition-value-solana) was applied
- The original CI from [Professional Education](https://github.com/solana-developers/professional-education) was re-added and updated to the latest version of Solana CLI, Anchor, node, etc.

## Mid 2024

This project was moved by me (when I worked at Solana Foundation) to [`program-examples`](https://github.com/solana-developers/program-examples). A second copy was stored in another repo at [`developer-bootcamp-2024`](https://github.com/solana-developers/developer-bootcamp-2024) which made maintenance more complex. However:

- Changes were made a couple of times made that broke tests.
- Other changes that favored performance over simplicity were made.
- Solana itself had changes that needed the repo(s) being updated.

## Early 2024

This project was created based on [Dean Little's Anchor Escrow](https://github.com/deanmlittle/anchor-escrow-2024), and stored in the [Solana Professional Education](https://github.com/solana-developers/professional-education) repo.

Compared to Dean's original, I made a few changes to make discussion in class easier:

One of the challenges when teaching is avoiding ambiguity — names have to be carefully chosen to be clear and not possible to confuse with other times.

- Custom instructions were replaced by `@solana-developers/helpers` for many tasks to reduce the file size.
- Shared functionality to transfer tokens is now in `instructions/shared.rs`
- The upstream project has a custom file layout. We use the 'multiple files' Anchor layout.
- Contexts are separate data structures from functions that use the contexts. There is no need for OO-like `impl` patterns here - there's no mutable state stored in the Context, and the 'methods' do not mutate that state. Besides, it's easier to type!
- The name 'deposit' was being used in multiple contexts, and `deposit` can be tough because it's a verb and a noun:

  - Renamed deposit #1 -> 'token_a_offered_amount'
  - Renamed deposit #2 (in make() ) -> 'send_offered_tokens_to_vault'
  - Renamed deposit #3 (in take() ) -> 'send_wanted_tokens_to_maker'

- 'seed' was renamed to 'id' because 'seed' as it conflicted with the 'seeds' used for PDA address generation.
- 'Escrow' was used for the program's name and the account that records details of the offer. This wasn't great because people would confuse 'Escrow' with the 'Vault'.

  - Escrow (the program) -> remains Escrow
  - Escrow (the offer) -> Offer.

- 'receive' was renamed to 'wanted_amount' as 'receive' is a verb and not a suitable name for an integer.
- mint_a -> offered_token, (ie, what the maker has offered and what the taker wants). We get this from the vault where necessary.
- mint_b -> wanted_token (ie, what that maker wants and what the taker must offer)
- makerAtaA -> makerTokenAccountA,
- makerAtaB -> makerTokenAccountB
- takerAtaA -> takerTokenAccountA
- takerAtaB -> takerTokenAccountB
