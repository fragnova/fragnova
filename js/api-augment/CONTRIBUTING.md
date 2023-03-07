# CONTRIBUTING

TODO

## Custom RPC Updates

CI will build everything from metadata except the custom RPCs. These are stored in `js/api-augment/interfaces/[pallet]/definitions.ts`.
If you add a new pallet, don't forget to also add the new definitions file export to `js/api-augment/interfaces/definitions.ts`.

## Useful Links

- [Type Definitions](https://github.com/polkadot-js/api/blob/master/packages/types/src/types/definitions.ts)

## Running Tests

Tests require getting the metadata and building first.

### Chain is running
- `js/api-augment` folder
- `npm run fetch:local` Fetches the metadata from localhost
- `npm run build`
