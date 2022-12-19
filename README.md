NEAR Social Avatars
======

Generative Composable NFT Avatars with a marketplace for avatar components provided by independent artists.

Live Demo: https://near.social/#/zavodil.near/widget/social-avatar-editor

Why it is cool:
===
- Onchain avatar generation & storage
- Avatar is transferable and tradable NFT
- NFT minted using Near.Social data only
- NFT artists earn tokens when someone is minting avatars with their artwork
- Avatars are made up of lightweight Vector Graphics (svg) files, so it's easy to add new avatar components
- Avatar components can be scarce and only added for a limited time, the artist controls all policies with his Near.Social storage

How it works
==
Storage:
- All Avatar components are stored on NEAR Social ([explorer](https://near.social/#/zavodil.near/widget/Explorer?path=zavodil.near/avtr/components/**/)), same for available colors ([explorer](https://near.social/#/zavodil.near/widget/Explorer?path=zavodil.near/avtr/colors/**)) 
- NFT Artists upload their components to Near.Social and indicate their desired price ([explorer](https://near.social/#/zavodil.near/widget/Explorer?path=vadim.near/avtr/components/**/))
 
Mint:
- User is selecting Social Avatar components on the [widget](https://near.social/#/zavodil.near/widget/social-avatar-editor)
- Avatar price is the sum of components parts (can be free) and NFT storage price (~0.2 NEAR)
- During the NFT mint, NFT Contract:
  - receives the list of chosen NFT components ([receipt](https://explorer.near.org/transactions/5LufhEzKKsUNLriktewCk1SBkywXr9LNfaq36iTPssgK#FKj1fEZJBYP9dGomLnxJoTqK7toby5D6XuYaazEf8wSh))
  - requests source values of each component and color from the Social DB ([receipt](https://explorer.near.org/transactions/5LufhEzKKsUNLriktewCk1SBkywXr9LNfaq36iTPssgK#CxN95S5d2n8f8gNkxwhJVUABz7WpzLDpVnYxn6tPgJNM))
  - generates svg graphics and stores in NFT media 
  - transfers the cost of the components to the creators of the corresponding components
  - sends graph notifications to Near.Social to inform creators about the purchase ([receipt](https://explorer.near.org/transactions/5LufhEzKKsUNLriktewCk1SBkywXr9LNfaq36iTPssgK#YRchrDnZQTaxr193feKNo1dUTamn31Kwbrgjoj6mGhd))

Widgets:
- [Avatar Generator](https://near.social/#/zavodil.near/widget/social-avatar-editor)
- [Avatar image viewer](https://near.social/#/zavodil.near/widget/social-avatar-image)
- [Owned avatars gallery](https://near.social/#/zavodil.near/widget/owned-social-avatars)
- [Add avatar components](https://near.social/#/zavodil.near/widget/add-avatar-component)
    
