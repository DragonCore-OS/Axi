use axi::core::{genesis::GenesisBlock, burn::Burn};
use axi::bridge::timelock::TimeLock;
use chrono::Utc;

#[tokio::main]
async fn main() {
    println!("⚡ AXI Node v{}", axi::VERSION);
    
    let args: Vec<String> = std::env::args().collect();
    
    match args.get(1).map(|s| s.as_str()) {
        Some("genesis") => {
            let block = GenesisBlock::new(1000.0, 3280.0);
            println!("Genesis Block:");
            println!("  Hash: {}", block.hash);
            println!("  Constitution: {}", block.constitution_hash);
            println!("  Power Anchor: {} kWh", block.anchor_power);
            println!("  Compute Anchor: {} TFLOPs", block.anchor_compute);
        }
        
        Some("status") => {
            let now = Utc::now().timestamp() as u64;
            let state = TimeLock::check(now);
            let days = TimeLock::days_until_independence(now);
            
            match state {
                axi::bridge::timelock::BridgeState::DualTrack => {
                    println!("Status: Dual-Track (Fiat allowed)");
                    println!("Days until Independence: {}", days);
                }
                axi::bridge::timelock::BridgeState::Sovereign => {
                    println!("Status: Sovereign (Physical anchor only)");
                }
            }
        }
        
        Some("wallet") => {
            use axi::wallet::key::KeyPair;
            let kp = KeyPair::generate();
            println!("New Wallet:");
            println!("  Address: {}", kp.address_string());
        }
        
        Some("burn-check") => {
            let now = Utc::now().timestamp() as u64;
            let last_move = now - (4 * 365 * 24 * 3600);
            let balance = 1000;
            
            let burn = Burn::calculate_burn(last_move, now, balance);
            println!("Balance: {} AXI", balance);
            println!("Dormant: 4 years");
            println!("Burn amount: {} AXI", burn);
            println!("Remaining: {} AXI", balance - burn);
        }
        
        Some("mint") => {
            println!("⚡ Genesis Mint: 13280 AXI");
            println!("  To: 0xf743080f5a30d59dd6167b4707280b9e1e300b8ca891689d496cba22882d2893");
            println!("  Proof: 1000kWh + 3280TFLOPs");
            println!("  Status: CONFIRMED");
            println!("  Note: This is a genesis record. No actual UTXO implemented yet.");
        }
        
        _ => {
            println!("Usage:");
            println!("  axi genesis      - Create genesis block");
            println!("  axi status       - Check Independence Day countdown");
            println!("  axi wallet       - Generate new wallet");
            println!("  axi burn-check   - Test halflife burn mechanism");
            println!("  axi mint         - Genesis mint record");
        }
    }
}
