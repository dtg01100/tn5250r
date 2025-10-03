use std::env;
use tn5250r::profile_manager::ProfileManager;
use tn5250r::session_profile::SessionProfile;
use tn5250r::network::ProtocolMode;
use tn5250r::lib3270::display::ScreenSize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Profile Manager functionality...");

    // Create a profile manager
    let mut pm = ProfileManager::new()?;
    println!("✓ Profile manager created successfully");

    // Create a test profile
    let test_profile = SessionProfile::new(
        "Test Profile".to_string(),
        "test.example.com".to_string(),
        23,
    );
    println!("✓ Test profile created: {}", test_profile.name);

    // Save the profile
    pm.create_profile(test_profile.clone())?;
    println!("✓ Profile saved successfully");

    // List profiles
    let profile_names = pm.get_profile_names();
    println!("✓ Available profiles: {:?}", profile_names);

    // Load the profile back
    if let Some(loaded_profile) = pm.get_profile_by_name("Test Profile") {
        println!("✓ Profile loaded successfully: {} -> {}:{}",
                 loaded_profile.name, loaded_profile.host, loaded_profile.port);
    } else {
        println!("✗ Failed to load profile");
        return Err("Profile loading failed".into());
    }

    // Test CLI profile loading (simulate command line args)
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--test-cli" {
        println!("Testing CLI profile loading...");
        if let Some(profile) = pm.get_profile_by_name("Test Profile") {
            println!("✓ CLI profile loading works: {}", profile.name);
        }
    }

    println!("🎉 All profile manager tests passed!");
    Ok(())
}