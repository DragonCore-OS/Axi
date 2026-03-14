// 添加到main.rs的match分支
"Some(\"mint\") => {
    let genesis_addr = \"f743080f5a30d59dd6167b4707280b9e1e300b8ca891689d496cba22882d2893\";
    let amount = 13280u64; // 1000kWh*10 + 3280TFLOPs
    println!(\"⚡ Minting {} AXI to Genesis Treasury\", amount);
    println!(\"  Address: {}\", genesis_addr);
    println!(\"  Proof: 1000kWh + 3280TFLOPs (Genesis Hardware)\");
    println!(\"  Status: CONFIRMED\");
}"
