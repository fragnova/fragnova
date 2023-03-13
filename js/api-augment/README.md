# `@fragnova/api-augment`

JavaScript/TypeScript SDK for interacting with the [Fragnova Blockchain](https://fragnova.com/)

<!-- GETTING STARTED -->
## Getting Started

- `npm install @fragnova/api-augment` (API Augmentation Library)
- `npm install @polkadot/api@9.14.2` (Polkadot API Library)

## Usage

For details on use, see the [Polkadot API library documentation](https://polkadot.js.org/docs/api).

```typescript
import { options } from "@fragnova/api-augment";
import { ApiPromise } from '@polkadot/api';
// ...

const api = await ApiPromise.create({
    provider: new WsProvider("ws://ws.fragnova.network"),
    ...options,
});
```
