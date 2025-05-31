use std::thread;
use std::time::Duration;
/// Example demonstrating Unicode character handling in UI interactions
///
/// This example shows how the library handles special characters from various languages.
///
/// To test:
/// 1. Open Notepad (or any text editor)
/// 2. Focus on the text area
/// 3. Run this example: cargo run --example unicode_test
/// 4. The program will wait 3 seconds, then automatically type test strings
use ui_interaction::core::AppendPosition;
use ui_interaction::create_automation;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("Unicode Character Test");
    println!("======================");
    println!("Please focus on a text input field...");
    println!("Testing will start in 3 seconds...");

    // Wait 3 seconds for user to focus on text field
    thread::sleep(Duration::from_secs(3));

    let automation = create_automation()?;

    // Comprehensive test strings with various international characters
    let test_cases = vec![
        ("ASCII", "Simple ASCII text 123"),
        (
            "French Extended",
            "Hôtel, naïve, coeur, œuf, façade, garçon",
        ),
        ("German", "Straße, größer, weiß, Mädchen, Müller, Bär, über"),
        ("Spanish", "niño, señor, año, corazón, mañana, peña"),
        ("Italian", "perché, città, così, più, università"),
        ("Portuguese", "ação, coração, não, João, Sáo Paulo"),
        ("Russian", "Привет мир! Москва, Россия, Français"),
        ("Japanese Hiragana", "こんにちは世界 (Hello World)"),
        ("Japanese Katakana", "コンピュータ テスト"),
        ("Japanese Kanji", "日本語 漢字 テスト"),
        ("Chinese", "你好世界 测试 中文"),
        ("Korean", "안녕하세요 세계 테스트"),
        ("Greek", "Γεια σας κόσμος δοκιμή αλφάβητο"),
        ("Arabic", "مرحبا بالعالم اختبار"),
        ("Hebrew", "שלום עולם בדיקה"),
        ("Thai", "สวัสดีครับ ทดสอบ"),
        ("Symbols", "€ £ ¥ © ® ™ ° ± × ÷ ≠ ≤ ≥"),
        ("Math Symbols", "∑ ∏ ∫ √ ∞ ∂ Δ Ω α β γ π"),
        ("Emojis", "🚀 🌟 💻 🎉 ⚡ 🔥 💡 ✨"),
        ("Mixed", "Café töörö мир 世界 🌍 Test123!"),
    ];

    for (category, test_text) in test_cases.iter() {
        println!("\n=== Testing {}: '{}' ===", category, test_text);

        // Get focused element
        let focused_element = automation.get_focused_element()?;

        // Clear the field
        focused_element.set_text("")?;
        thread::sleep(Duration::from_millis(200));

        // Add text with special characters
        focused_element.append_text(test_text, AppendPosition::EndOfText)?;

        // Wait for text to be processed
        thread::sleep(Duration::from_millis(500));

        // Get the result back
        let result_text = focused_element.get_text()?;

        println!("Expected: '{}'", test_text);
        println!("Got:      '{}'", result_text);

        if test_text == &result_text {
            println!("✓ PASSED - All characters preserved correctly");
        } else {
            println!("✗ FAILED - Character encoding issues detected");

            // Show character differences in detail
            let expected_chars: Vec<char> = test_text.chars().collect();
            let result_chars: Vec<char> = result_text.chars().collect();

            println!("Character-by-character analysis:");
            let max_len = expected_chars.len().max(result_chars.len());

            for i in 0..max_len {
                let expected = expected_chars.get(i).copied().unwrap_or('\0');
                let actual = result_chars.get(i).copied().unwrap_or('\0');

                if expected != actual {
                    println!(
                        "  [{}] Expected '{}' (U+{:04X}) → Got '{}' (U+{:04X})",
                        i, expected, expected as u32, actual, actual as u32
                    );
                }
            }
        }

        // Short pause between tests
        thread::sleep(Duration::from_millis(500));
    }

    println!("\n=== All tests completed! ===");
    println!("Check the results above to verify Unicode character handling.");
    Ok(())
}

