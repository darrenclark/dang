use strum::EnumCount;

use super::inst::OpCode;

#[derive(Debug)]
pub struct StatsCollector {
    counts: [usize; 256],
    nanos: [i64; 256],
}

impl StatsCollector {
    pub fn new() -> Self {
        Self {
            counts: [0; 256],
            nanos: [0; 256],
        }
    }
    pub fn record(&mut self, opcode: OpCode, nanos: i64) {
        self.counts[opcode as usize] += 1;
        self.nanos[opcode as usize] += nanos;
    }
    pub fn print(&self) {
        let mut rows = Vec::new();

        for i in 0..OpCode::COUNT {
            if self.counts[i] > 0 {
                let opcode = OpCode::from_repr(i as u8).unwrap();
                let nanos = self.nanos[i];
                let count = self.counts[i];
                let avg = nanos / count as i64;
                let msg = format!(
                    "{:?}: {}, {} times, avg {:?} ns/instr",
                    opcode,
                    Self::format_time(nanos),
                    count,
                    avg
                );
                rows.push((nanos, msg));
            }
        }

        rows.sort_by(|a, b| b.0.cmp(&a.0));

        for (_, s) in rows {
            println!("{}", s);
        }
    }

    fn format_time(nanos: i64) -> String {
        if nanos < 1000 {
            format!("{} ns", nanos)
        } else if nanos < 1_000_000 {
            format!("{:.1} us", nanos as f64 / 1_000.0)
        } else if nanos < 1_000_000_000 {
            format!("{:.1} ms", nanos as f64 / 1_000_000.0)
        } else {
            format!("{:.1} s", nanos as f64 / 1_000_000_000.0)
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct InstructionTimer {
    start: std::time::Instant,
    op: OpCode,
}

#[allow(dead_code)]
impl InstructionTimer {
    pub fn new(op: OpCode) -> Self {
        Self {
            start: std::time::Instant::now(),
            op,
        }
    }

    pub fn stop(&mut self, stats: &mut StatsCollector) {
        let elapsed = self.start.elapsed().as_nanos();
        stats.record(self.op, elapsed as i64);
    }
}
