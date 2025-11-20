//! Simple echo example demonstrating rRPC

use rrpc_core::{get_registry, rrpc_init, RpcError};

fn echo_handler(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    println!("Echo received: {} bytes", input.len());
    Ok(input.to_vec())
}

fn reverse_handler(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let mut result = input.to_vec();
    result.reverse();
    println!("Reversed {} bytes", result.len());
    Ok(result)
}

fn main() {
    println!("=== rRPC Echo Demo ===\n");
    
    // Initialize rRPC runtime
    unsafe {
        rrpc_init();
    }
    println!("✓ rRPC initialized");
    
    // Register handlers
    let registry = get_registry().unwrap();
    {
        let mut reg = registry.lock();
        reg.register("echo", echo_handler);
        reg.register("reverse", reverse_handler);
    }
    println!("✓ Registered 2 handlers: echo, reverse\n");
    
    // Test echo
    {
        let reg = registry.lock();
        let input = b"Hello, rRPC!";
        println!("Calling 'echo' with: {:?}", std::str::from_utf8(input).unwrap());
        
        match reg.call("echo", input) {
            Ok(output) => {
                println!("  Result: {:?}", std::str::from_utf8(&output).unwrap());
            }
            Err(e) => println!("  Error: {}", e),
        }
    }
    
    println!();
    
    // Test reverse
    {
        let reg = registry.lock();
        let input = b"Hello, rRPC!";
        println!("Calling 'reverse' with: {:?}", std::str::from_utf8(input).unwrap());
        
        match reg.call("reverse", input) {
            Ok(output) => {
                println!("  Result: {:?}", std::str::from_utf8(&output).unwrap());
            }
            Err(e) => println!("  Error: {}", e),
        }
    }
    
    println!();
    
    // Test unknown method
    {
        let reg = registry.lock();
        println!("Calling unknown method 'missing'...");
        match reg.call("missing", b"test") {
            Ok(_) => println!("  Unexpected success"),
            Err(e) => println!("  Expected error: {}", e),
        }
    }
    
    println!("\n=== Demo Complete ===");
}
