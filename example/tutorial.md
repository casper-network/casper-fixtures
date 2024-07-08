# Casper migrating from 1.5.6 to 2.0.0

## Changes relevant to Smart Contract development
Numerous changes have been made that need to be addressed inside smart contracts.
1. StoredSession entrypoints have been removed as a possibility and there are no replacements.
2. EntryPointPayment argument have been added to EntryPoint declarations,
that denote who pays for which parts during the entrypoints call:
    - Caller: pays for everything
    - SelfOnly: the contract pays for the entrypoints call
    - SelfOnward: the contract pays for the entrypoints call, and any subsecvent calls that the entrypoint makes
3. CallStackElement was changed to reflect changes.
4. Messages and MessageTopics were added. These are technically the native "events" on Casper.
Messages are not stored in global state, but are added to the ExecutionResult as a separete `Messages` field.
Since Messages are not added to the global state, they are approximately 1/10th the cost as writing data into dictionaries.
5. `AddressableEntity` was added to better reflect and access the global state, coupled with `AddressableEntityHash`. Creating a new contract version will return an `AddressableEntityHash` instead of `ContractHash`.
6. `PackageHash` was added, partially because the old `ContractPackageHash` was sometimes a bit confusing next to the `ContractHash` type. From now on deploying a brand new contract will create a `PackageHash` instead of a `ContractPackageHash`
7. `EntityAddr` a new enum with variants that contains:
    - `EntityAddr::Account(hash)`
    - `EntityAddr::SmartContract(hash)`
    - `EntityAddr::System(hash)`
8. The `Key` enum type was extended with multiple new variants, from smart contract development point of view and including the already existing ones the most important `Key` variants are:
    - `Key::Hash` in 1.5.6 this was used for both `ContractHash` and `ContractPackageHash` storage
    - `Key::Account` was used specifically for `AccountHash` storage.
    - `Key::Package` (new) is used specifically for storage of the new `PackageHash` type, to intentionally make it more discint, unlike what's seen with `Key::Hash`.
    - `Key::AddressableEntity` (new) which hold an `EntityAddr`.

## Precautions to make
The bigger precaution that needs to be known is that `Key`s and hash types are changed, so unless handled adequately, data tied to them can be lost in the contracts.

## Fixtures/Snapshots
There is a possibility in the Execution Engine tests to load global state from an lmdb database. This enables us to test create a global state version using 1.5.6, store it into lmdb, then switch our Execution Engine version to 2.0.0 and using that load the global state from lmdb and after upgrading the chainspec test our contracts upgrade/migration/usage on 2.0.0.
As a short list:
1. create our global state (deploy contracts and set them up as we need to)
2. save the global state into lmdb
3. shrink lmdb size (it is unnecessarily large)
4. load out lmdb global state into a 2.0.0 execution engine
5. upgrade the chainspec to 2.0.0 in the execution engine
6. test our contract, with or without upgrading/migrating the contract.