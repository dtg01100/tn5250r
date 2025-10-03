use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tn5250r::lib5250::protocol::{parse_5250_stream, ProtocolState};
use tn5250r::terminal::TerminalScreen;
use tn5250r::field_manager::FieldType;

// Mock implementation for benchmarking
struct MockProtocolState {
    screen: TerminalScreen,
}

impl MockProtocolState {
    fn new() -> Self {
        Self {
            screen: TerminalScreen::new(),
        }
    }
}

impl ProtocolState for MockProtocolState {
    fn set_cursor(&mut self, row: usize, col: usize) {
        self.screen.set_cursor(row, col);
    }

    fn add_field(&mut self, row: usize, col: usize, length: usize, field_type: FieldType, attribute: u8) {
        // Simplified - just track the field
        let _ = (row, col, length, field_type, attribute);
    }

    fn determine_field_type(&mut self, attribute: u8) -> FieldType {
        if attribute & 0x20 != 0 {
            FieldType::Protected
        } else {
            FieldType::Input
        }
    }

    fn detect_fields(&mut self) {
        // No-op for benchmark
    }

    fn screen(&mut self) -> &mut TerminalScreen {
        &mut self.screen
    }
}

fn bench_parse_5250_stream(c: &mut Criterion) {
    // Sample 5250 data: WriteToDisplay command with some fields
    let data = vec![
        0xF1, // WriteToDisplay
        0x00, // WCC
        0x11, 0x20, // Field attribute
        0x1A, 0x01, 0x01, // Set cursor to (0,0)
        0xC8, 0xE4, 0xD5, 0xC3, 0xD6, 0xD4, 0xC5, // "TN5250R" in EBCDIC
        0xF0, // End of command
    ];

    c.bench_function("parse_5250_stream", |b| {
        b.iter(|| {
            let mut state = MockProtocolState::new();
            black_box(parse_5250_stream(black_box(&data), black_box(&mut state))).unwrap();
        })
    });
}

criterion_group!(benches, bench_parse_5250_stream);
criterion_main!(benches);