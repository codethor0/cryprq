# Pack 3: Error Handling & Validation - Implementation Status

##  Completed

### 1. Central Error Catalog
-  `gui/src/errors/catalog.ts` with all error codes
-  Human-readable titles, descriptions, help text
-  Doc slug placeholders for future documentation
-  Error mapping function `mapErrorToCode()`

### 2. Validation Utilities
-  `gui/src/utils/validation.ts` with:
  - `isValidPort()` - Port range validation (1-65535)
  - `isValidRotationMinutes()` - Minimum 1 minute
  - `parseAndValidateMultiaddr()` - Multiaddr format validation
  - Help text constants for tooltips

### 3. Error Bus System
-  `gui/src/hooks/useErrorBus.ts` - Central error event bus
-  Subscribes to backend session errors
-  Maps low-level errors to CrypRQErrorCode
-  Integrates with store for logging

### 4. Error Surfaces

**Toast System:**
-  `gui/src/components/Toast/Toaster.tsx` - Toast component
-  `gui/src/store/toastStore.ts` - Toast state management
-  Auto-dismiss after 6 seconds
-  Max 3 concurrent toasts
-  Bottom-right positioning

**Error Modal:**
-  `gui/src/components/ErrorModal/ErrorModal.tsx` - Blocking modal
-  Shows error title, description, help text
-  Expandable error details (last logs)
-  Remediation CTAs:
  - PORT_IN_USE → "Pick Another Port" (navigates to Settings)
  - CLI_NOT_FOUND → "Locate Binary..." (file picker placeholder)
  - CLI_EXITED → "Restart Session" + "View Logs"

### 5. Error Propagation
-  Backend service maps errors to CrypRQErrorCode
-  Error bus emits errors to UI components
-  App.tsx routes blocking vs non-blocking errors
-  Toast for transient errors (METRICS_TIMEOUT)
-  Modal for critical errors (PORT_IN_USE, CLI_NOT_FOUND, etc.)

## ⏳ Pending (Pack 3)

### Form Validation
- ⏳ Settings component: Inline validation for port, rotation interval
- ⏳ Settings component: Block Save button on validation errors
- ⏳ Peers component: Multiaddr validation in Add Peer dialog
- ⏳ Peers component: Disable "Add" button until valid
- ⏳ Field-level error messages with help text

### Additional Features
- ⏳ "Test reachability" button in Add Peer modal
- ⏳ IPC handler `peer:testReachability`
- ⏳ Tooltip component for field help text
- ⏳ Field focus management (Settings port field)

## Testing Checklist

### Acceptance Tests
- [ ] Enter port 70000 → inline error blocks Save; toast NOT shown
- [ ] Start when port is in use → modal with "Pick another port" CTA
- [ ] Click "Pick another port" → navigates to Settings and focuses port field
- [ ] After changing port, Connect succeeds
- [ ] Bad multiaddr in Add Peer → button disabled + inline helper
- [ ] Kill the CLI → "Session ended unexpectedly" modal with "Restart session" and "View logs"
- [ ] Restart works after error modal
- [ ] Simulate metrics host down → toast "Metrics temporarily unavailable" (no modal)

## Next Steps

1. **Add form validation to Settings component**
   - Validate port on change
   - Validate rotation interval on change
   - Show inline errors
   - Disable Save button when errors exist

2. **Add form validation to Peers component**
   - Validate multiaddr format
   - Show inline error message
   - Disable Add button until valid

3. **Add tooltip component**
   - Accessible tooltip with delay
   - Add to rotation interval, multiaddr fields

4. **Add reachability test**
   - IPC handler in electron/main
   - Button in Add Peer modal
   - Show spinner → success/error feedback

