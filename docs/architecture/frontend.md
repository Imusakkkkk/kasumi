# Qt Frontend

- `gui-qt/src/main.cpp`: splash screen, app metadata (name "oboromi").
- `MainWindow`, `RustBridge`: FFI communication.
- Qt resources: `resources.qrc` (logo).
- Startup: fade-in splash → MainWindow.

Debugging/logging basic. Separation between GUI and core maintained via FFI.