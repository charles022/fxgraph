Updates performed:
- Backend: ran `cargo update`, refreshing `Cargo.lock` to latest compatible crate versions.
- Frontend: bumped dependencies/devDependencies to current compatible releases (React 18.3, Connect 1.7/protobuf 1.10, Buf CLI 1.61, lucide 0.555, Tailwind 3.4.16, TS 5.6.3), cleaned `node_modules` and reinstalled, regenerated `package-lock.json`, and rebuilt generated client code with `npm run generate`.
- Verified `npm run lint` (frontend) passes.

Possible next steps:
- Run full app build/tests (`npm run build`, backend tests) to confirm runtime compatibility.
- Consider future upgrade path to Connect/Buf v2 when ready to migrate generator/transport APIs.
- Clean the workspace with `./scripts/clean.sh` if needed after additional work.
