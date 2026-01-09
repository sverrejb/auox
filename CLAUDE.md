# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Auox (Aurum Oxydatum) is a terminal-based banking application written in Rust that interfaces with SpareBank 1's API. It provides a TUI (Terminal User Interface) for viewing and managing bank accounts using OAuth authentication.

## Commands

### Build and Run
```bash
cargo build          # Build the project
cargo run            # Run the application
cargo check          # Quick type checking without building
cargo clippy         # Run linter
```

### Testing
```bash
cargo test           # Run all tests
cargo test <name>    # Run a specific test
```

## Architecture

### Module Structure
- `src/main.rs`: Application entry point and TUI event loop
- `src/auth.rs`: OAuth authentication flow implementation
- `src/fileio.rs`: Configuration and token file management
- `src/api.rs`: SpareBank 1 API client functions
- `src/transfers.rs`: Transfer business logic
- `src/models/`: Data models for API responses
  - `accounts.rs`: Account data structures
  - `token.rs`: Token data structures
  - `transactions.rs`: Transaction data structures
  - `transfers.rs`: Transfer DTOs and response structures
- `src/ui.rs`: TUI rendering logic

### OAuth Authentication Flow
The application implements a three-tiered OAuth authentication strategy in `src/auth.rs`:

1. **Access Token Check**: First attempts to use stored access token from `auth.json`
2. **Refresh Token Flow**: If access token invalid, attempts to refresh using stored refresh token
   - Makes POST request to `/oauth/token` with `grant_type=refresh_token`
   - Saves new token data to `auth.json` on success
3. **Full OAuth Flow**: If refresh fails, initiates full OAuth flow with SpareBank 1's API:
   - Spawns local HTTP server on port 8321 to receive OAuth callback
   - Opens browser to SpareBank 1's authorization endpoint with `finInst` parameter from config
   - Waits for authorization code via redirect
   - Exchanges code for access token

### File Management (`src/fileio.rs`)
- **Config file location**: `~/.config/auox/config.toml` (on macOS/Linux) or equivalent platform directory
- **Token file location**: `~/.local/share/auox/auth.json` (on Linux) or equivalent platform data directory
- Creates directories and config template automatically on first run if they don't exist
- **Required config fields**:
  - `client_id`: OAuth client ID for SpareBank 1 API
  - `client_secret`: OAuth client secret for SpareBank 1 API
  - `financial_institution`: Financial institution ID (e.g., `fid-smn` for SpareBank 1 Midt-Norge)
- **Token file structure** (`auth.json`):
  - `access_token`: Current access token
  - `expires_in`: Access token expiry time (seconds)
  - `refresh_token`: Token for refreshing access
  - `refresh_token_expires_in`: Refresh token expiry time
  - `refresh_token_absolute_expires_in`: Absolute refresh token expiry
  - `token_type`: Token type (Bearer)

### TUI Architecture (`src/main.rs`, `src/ui.rs`)
- Built with `ratatui` and `crossterm` for terminal UI
- Main loop pattern: enable raw mode → render loop → cleanup
- **Multiple views**:
  - `Accounts`: Main account list view
  - `Menu`: Action menu for selected account
  - `Transactions`: Transaction history for selected account
  - `TransferSelect`: Select destination account for transfer
  - `TransferModal`: Enter transfer amount and optional message
- **Event handling**:
  - `Ctrl+C`: Exit application with dissolve animation effect
  - `Esc`: Navigate back in view stack
  - `Enter`: Select item / confirm action
  - `Up/Down arrows`: Navigate lists with modulo wrap-around
  - `b`: Toggle balance visibility
  - `m`: Toggle credit card visibility
  - `Tab`: Switch between input fields (in transfer modal)
- State management via `TableState` and `ListState` for tracking selections
- UI rendered in `ui::draw()` with blue highlight style for selected items
- Visual effects powered by `tachyonfx` (coalesce in, dissolve out)

### API Client (`src/api.rs`)
- `get_accounts()`: Fetches account list from `/personal/banking/accounts`
- `get_transactions(account_key)`: Fetches transaction history from `/personal/banking/transactions`
- `create_transfer(transfer)`: Creates regular account transfer via `/personal/banking/transfer/debit`
- `create_credit_card_transfer(transfer)`: Creates credit card transfer via `/personal/banking/transfer/creditcard/transferTo`
- `perform_transfer(app)`: High-level transfer logic that automatically detects credit cards and routes to appropriate endpoint
- Uses Bearer token authentication
- Returns structured response types with error handling

### Data Models (`src/models/`)
Defines SpareBank 1 API response structures:
- **accounts.rs**:
  - `AccountData`: Top-level response with accounts array and errors
  - `Account`: Bank account with balance, IBAN, owner info, account properties, and credit card ID
  - `AccountProperties`: Detailed flags for account capabilities (transfers, payments, special account types)
- **transactions.rs**:
  - `TransactionResponse`: Top-level response with transactions array and errors
  - `Transaction`: Individual transaction with amount, date, description, type, and optional KID/message
- **transfers.rs**:
  - `CreateTransferDTO`: Request DTO for regular account transfers (includes optional message field)
  - `TransferToCreditCardDTO`: Request DTO for credit card transfers (uses credit card ID instead of account number)
  - `TransferResponse`: Response with payment ID, status, and structured error details
  - `ErrorDTO`: Detailed error information with code, message, trace ID, and localized messages
- **token.rs**:
  - `TokenData`: OAuth token response structure
- All structs use camelCase JSON serialization to match API format

## Key Dependencies
- `ratatui`: TUI framework for terminal interface
- `crossterm`: Cross-platform terminal manipulation
- `tachyonfx`: Visual effects for TUI (coalesce, dissolve animations)
- `tui-input`: Text input widgets for forms
- `tiny_http`: Lightweight HTTP server for OAuth callback
- `reqwest`: HTTP client for API calls (with `blocking` and `json` features enabled)
- `serde`/`serde_json`/`toml`: Serialization and config parsing
- `color-eyre`: Enhanced error reporting
- `dirs`: Platform-agnostic directory paths
- `log`/`env_logger`: Logging framework
- `chrono`: Date/time formatting for transactions

## Development Notes

### Current State
The application is functional with core features implemented:
- ✅ OAuth authentication with token refresh
- ✅ Token storage and retrieval from filesystem
- ✅ Account fetching from SpareBank 1 API (including credit cards)
- ✅ Transaction history viewing
- ✅ Account-to-account transfers with optional message
- ✅ Credit card transfers with automatic detection
- ✅ TUI with multiple views (accounts, menu, transactions, transfers)
- ✅ Keyboard navigation with shortcuts
- ✅ Visual effects and animations
- ✅ Balance and credit card visibility toggles

### Incomplete Features
- `get_access_token()` in `src/auth.rs` - needs implementation to exchange OAuth code for tokens (full OAuth flow completion)
- Account details view - currently only shows list, no detailed view when selecting an account
- Error handling - many functions use `panic!` or `expect()` instead of proper error propagation
- User feedback on transfer success/failure - currently only logged, not shown in UI
- File-based logging - logs currently go to stderr which breaks the TUI (println! statements removed from transfer code)
- Payment functionality (bills, invoices) - not yet implemented
- Transaction filtering and search
- Export functionality (CSV, PDF)
- Settings/preferences management

### API Integration
- Base URL: `https://api.sparebank1.no`
- OAuth endpoint: `/oauth/authorize`
- Financial institution parameter: `finInst` (configurable, e.g., `fid-smn` for SpareBank 1 Midt-Norge)
- Redirect URI: `http://localhost:8321`
- **API Endpoints**:
  - `/personal/banking/accounts` - GET account list
  - `/personal/banking/transactions?accountKey={key}` - GET transaction history
  - `/personal/banking/transfer/debit` - POST regular account transfer
  - `/personal/banking/transfer/creditcard/transferTo` - POST credit card transfer

### Transfer Functionality
The application supports two types of transfers with automatic routing:

#### Regular Account Transfers
- **Endpoint**: `/personal/banking/transfer/debit`
- **DTO**: `CreateTransferDTO`
- **Fields**:
  - `amount`: String (0.01 - 1000000000)
  - `fromAccount`: Source account number
  - `toAccount`: Destination account number
  - `message`: Optional message (max 40 characters)
  - `dueDate`: Optional due date (defaults to today)
  - `currencyCode`: Optional (defaults to source account currency)

#### Credit Card Transfers
- **Endpoint**: `/personal/banking/transfer/creditcard/transferTo`
- **DTO**: `TransferToCreditCardDTO`
- **Fields**:
  - `amount`: String (0.01 - 999999999)
  - `fromAccount`: Source account number
  - `creditCardAccountId`: Credit card ID (not account number)
  - `dueDate`: Optional due date (defaults to today)
- **Note**: Credit card transfers do NOT support the message field

#### Automatic Detection
The `perform_transfer()` function in `src/api.rs` automatically:
1. Checks if destination account has `type_field == "CREDITCARD"`
2. Routes to appropriate endpoint and uses correct DTO
3. For credit cards: uses `credit_card_account_id` field from Account model
4. For regular accounts: uses `account_number` field

#### UI Flow
1. User selects "Transfer from" in account menu → selects source account
2. User selects destination account from account list
3. Transfer modal appears with:
   - Amount input (always visible)
   - Message input (always visible, but ignored for credit card transfers)
   - Tab key switches between inputs
   - Active input highlighted in yellow, inactive in gray
4. Enter key executes transfer
5. On success: returns to account list with refreshed balances
6. On error: logs detailed error information with trace IDs
