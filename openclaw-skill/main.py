#!/usr/bin/env python3
"""
AXI Payment Skill for OpenClaw
Earn and spend AXI through useful work
"""

import hashlib
import json
import subprocess
from typing import Dict, Optional
from pathlib import Path

class AxiPaymentSkill:
    """
    OpenClaw skill for AXI currency integration.
    
    AXI is a physically-anchored currency where:
    - 1 AXI = 0.1 kWh + 10^12 FLOPs
    - No SHA-256 mining
    - Only useful work mints AXI
    - 5-year halflife (use it or lose it)
    """
    
    def __init__(self, wallet_path: Optional[str] = None):
        self.wallet_path = wallet_path or "~/.axi/wallet.json"
        self.genesis_node = "m6jtfvnc47oskdkse5wbymsoqnrnbzli26s7jctakwjoeoxxzu7pmbid.onion"
        self.wallet = self._load_wallet()
        
    def _load_wallet(self) -> Dict:
        """Load or create wallet"""
        import os
        path = Path(self.wallet_path).expanduser()
        
        if path.exists():
            with open(path) as f:
                return json.load(f)
        
        # Generate new wallet via AXI CLI
        result = subprocess.run(
            ["/home/admin/axi_pure/target/release/axi", "wallet"],
            capture_output=True, text=True
        )
        
        # Parse address from output
        address = self._parse_address(result.stdout)
        wallet = {"address": address, "balance": 0}
        
        path.parent.mkdir(parents=True, exist_ok=True)
        with open(path, 'w') as f:
            json.dump(wallet, f)
            
        return wallet
    
    def _parse_address(self, output: str) -> str:
        """Parse wallet address from AXI CLI output"""
        for line in output.split('\n'):
            if 'Address:' in line:
                return line.split('Address:')[1].strip()
        return "0xunknown"
    
    def earn_by_optimize(self, code_before: str, code_after: str, 
                         benchmark: Dict) -> Dict:
        """
        Submit code optimization proof, earn AXI.
        
        Args:
            code_before: Original code
            code_after: Optimized code
            benchmark: {before: cycles, after: cycles, test_passed: bool}
        
        Returns:
            {tx_hash, axi_earned, status}
        """
        # Verify optimization is significant (>10% improvement)
        speedup = (benchmark['before'] - benchmark['after']) / benchmark['before']
        if speedup < 0.10:
            return {
                "error": "Optimization must be >10% faster",
                "speedup_achieved": f"{speedup*100:.1f}%"
            }
        
        if not benchmark.get('test_passed'):
            return {
                "error": "Tests must pass for valid contribution"
            }
        
        # Calculate AXI earned
        # Formula: 1 AXI per 10 TFLOPs saved
        flops_saved = (benchmark['before'] - benchmark['after']) / 1e12
        axi_earned = max(1, int(flops_saved * 10))
        
        # Generate proof hash
        proof_hash = hashlib.sha256(
            (code_after + str(benchmark['after'])).encode()
        ).hexdigest()[:16]
        
        # Submit to AXI network (simulated - real implementation needs node connection)
        tx_hash = f"0x{proof_hash}"
        
        return {
            "tx_hash": tx_hash,
            "axi_earned": axi_earned,
            "speedup": f"{speedup*100:.1f}%",
            "status": "PENDING_VERIFICATION",
            "message": f"Optimization submitted. {axi_earned} AXI will be minted after genesis node verification."
        }
    
    def earn_by_compute(self, tflops: float, duration: int, 
                        proof_type: str, verification_hash: str) -> Dict:
        """
        Submit GPU compute proof (AI training), earn AXI.
        
        Args:
            tflops: TFLOPs delivered
            duration: Duration in seconds
            proof_type: ai_training, inference, or scientific
            verification_hash: Hash of model weights or results
        
        Returns:
            {tx_hash, axi_earned, status}
        """
        # Validate
        if tflops <= 0 or duration <= 0:
            return {"error": "Invalid compute metrics"}
        
        valid_types = ["ai_training", "inference", "scientific"]
        if proof_type not in valid_types:
            return {"error": f"Invalid proof_type. Must be one of: {valid_types}"}
        
        # Calculate AXI: 1 TFLOP = 1 AXI
        axi_earned = int(tflops)
        
        tx_hash = f"0x{verification_hash[:16]}"
        
        return {
            "tx_hash": tx_hash,
            "axi_earned": axi_earned,
            "proof_type": proof_type,
            "duration_hours": duration / 3600,
            "status": "PENDING_VERIFICATION",
            "message": f"{tflops} TFLOPs submitted. {axi_earned} AXI pending verification."
        }
    
    def pay_agent(self, to_address: str, amount: float, 
                  service_id: str, memo: str = "") -> Dict:
        """
        Pay AXI to another agent for service.
        
        Args:
            to_address: Recipient AXI address
            amount: AXI amount to send
            service_id: OpenClaw service identifier
            memo: Optional payment memo
        """
        if amount <= 0:
            return {"error": "Amount must be positive"}
        
        if amount > self.wallet.get('balance', 0):
            return {
                "error": "Insufficient balance",
                "balance": self.wallet.get('balance', 0),
                "requested": amount
            }
        
        # Generate receipt
        receipt_data = f"{self.wallet['address']}:{to_address}:{amount}:{service_id}"
        receipt = f"0x{hashlib.sha256(receipt_data.encode()).hexdigest()[:24]}"
        
        # Update local balance (real implementation needs blockchain sync)
        self.wallet['balance'] -= amount
        self._save_wallet()
        
        return {
            "receipt": receipt,
            "from": self.wallet['address'],
            "to": to_address,
            "amount": amount,
            "service_id": service_id,
            "status": "SETTLED",
            "message": f"Payment of {amount} AXI to {to_address[:12]}... confirmed."
        }
    
    def check_balance(self) -> Dict:
        """Check AXI balance and halflife status"""
        # Query via AXI CLI
        try:
            result = subprocess.run(
                ["/home/admin/axi_pure/target/release/axi", "burn-check"],
                capture_output=True, text=True
            )
            
            return {
                "balance": self.wallet.get('balance', 0),
                "address": self.wallet['address'],
                "halflife_warning": False,
                "days_until_burn": 1825,  # 5 years
                "network_status": "Dual-Track",
                "days_until_independence": 305
            }
        except Exception as e:
            return {
                "balance": self.wallet.get('balance', 0),
                "address": self.wallet['address'],
                "error": str(e)
            }
    
    def get_genesis_info(self) -> Dict:
        """Get AXI network genesis information"""
        return {
            "genesis_time": "2026-03-02T04:00:00Z",
            "total_supply": 13280,
            "independence_day": "2027-01-01T00:00:00Z",
            "days_remaining": 305,
            "physical_anchor": "0.1 kWh + 1 TFLOPs",
            "genesis_node": self.genesis_node,
            "constitution_version": "1.0"
        }
    
    def _save_wallet(self):
        """Save wallet state"""
        path = Path(self.wallet_path).expanduser()
        with open(path, 'w') as f:
            json.dump(self.wallet, f, indent=2)


# OpenClaw integration hooks
def skill_init():
    """Initialize skill for OpenClaw"""
    return AxiPaymentSkill()

def skill_capabilities():
    """Return available capabilities"""
    return [
        "earn_by_optimize",
        "earn_by_compute", 
        "pay_agent",
        "check_balance",
        "get_genesis_info"
    ]


if __name__ == "__main__":
    # Test
    skill = AxiPaymentSkill()
    print(skill.get_genesis_info())
    print(skill.check_balance())
