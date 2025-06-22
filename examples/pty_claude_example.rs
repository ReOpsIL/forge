// Example of using Claude Code with proper PTY support
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct ClaudeSession {
    pub pty_pair: Box<dyn portable_pty::PtyPair>,
    pub child: Box<dyn portable_pty::Child>,
}

impl ClaudeSession {
    pub fn new(working_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let pty_system = native_pty_system();
        
        // Create a new pty session
        let pty_pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Setup Claude command with proper environment
        let mut cmd = CommandBuilder::new("claude");
        cmd.arg("--dangerously-skip-permissions");
        cmd.cwd(working_dir);
        
        // Set environment variables for better Claude behavior
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLUMNS", "80");
        cmd.env("LINES", "24");
        
        // Spawn Claude in the pty
        let child = pty_pair.slave.spawn_command(cmd)?;
        
        Ok(ClaudeSession { pty_pair, child })
    }

    pub fn execute_task_with_feedback(
        &mut self,
        task_prompt: &str,
    ) -> Result<TaskExecutionResult, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel();
        
        // Clone master for reading
        let mut reader = self.pty_pair.master.try_clone_reader()?;
        let mut writer = self.pty_pair.master.take_writer()?;
        
        // Spawn thread to read Claude's output
        let tx_clone = tx.clone();
        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            let mut accumulated_output = String::new();
            
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buf[..n]);
                        accumulated_output.push_str(&text);
                        
                        // Send output chunks
                        let _ = tx_clone.send(OutputEvent::Data(text.to_string()));
                        
                        // Check for completion indicators
                        if let Some(result) = parse_completion_indicators(&accumulated_output) {
                            let _ = tx_clone.send(OutputEvent::Completion(result));
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        
        // Send the task prompt
        writer.write_all(task_prompt.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        
        // Wait for completion with timeout
        let mut execution_result = TaskExecutionResult::default();
        let timeout = Duration::from_secs(300); // 5 minutes
        let start_time = std::time::Instant::now();
        
        while start_time.elapsed() < timeout {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(OutputEvent::Data(data)) => {
                    execution_result.output.push_str(&data);
                    execution_result.last_activity = std::time::Instant::now();
                }
                Ok(OutputEvent::Completion(result)) => {
                    execution_result.completed = true;
                    execution_result.success = result.success;
                    execution_result.files_modified = result.files_modified;
                    execution_result.summary = result.summary;
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Check if Claude is still active
                    if start_time.elapsed() > Duration::from_secs(30) && 
                       execution_result.last_activity.elapsed() > Duration::from_secs(30) {
                        // Claude seems stuck, send a gentle prompt
                        writer.write_all(b"\nAre you finished with this task? Please confirm completion status.\n")?;
                        writer.flush()?;
                    }
                }
                Err(_) => break,
            }
        }
        
        Ok(execution_result)
    }
}

#[derive(Debug, Clone)]
pub enum OutputEvent {
    Data(String),
    Completion(CompletionResult),
}

#[derive(Debug, Clone, Default)]
pub struct TaskExecutionResult {
    pub completed: bool,
    pub success: bool,
    pub output: String,
    pub files_modified: Vec<String>,
    pub summary: String,
    pub last_activity: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct CompletionResult {
    pub success: bool,
    pub files_modified: Vec<String>,
    pub summary: String,
}

// Parse Claude's output for completion indicators
fn parse_completion_indicators(output: &str) -> Option<CompletionResult> {
    // Look for common completion patterns in Claude's output
    let patterns = [
        // Claude Code specific patterns
        r"✅.*(?:completed?|finished?|done)",
        r"🎉.*(?:task.*complete|implementation.*finished)",
        r"✨.*(?:successfully.*implemented|task.*accomplished)",
        
        // Git-related completion indicators
        r"Successfully committed.*",
        r"Changes have been.*committed",
        r"Implementation complete.*",
        
        // Error patterns
        r"❌.*(?:failed?|error|unable)",
        r"🚨.*(?:issue|problem|failed)",
    ];
    
    let lower_output = output.to_lowercase();
    
    // Check for success patterns
    for pattern in &patterns[..6] { // First 6 are success patterns
        if regex::Regex::new(pattern).unwrap().is_match(&lower_output) {
            return Some(CompletionResult {
                success: true,
                files_modified: extract_modified_files(output),
                summary: extract_summary(output),
            });
        }
    }
    
    // Check for error patterns
    for pattern in &patterns[6..] { // Last patterns are error patterns
        if regex::Regex::new(pattern).unwrap().is_match(&lower_output) {
            return Some(CompletionResult {
                success: false,
                files_modified: extract_modified_files(output),
                summary: extract_summary(output),
            });
        }
    }
    
    // Check for common completion phrases
    if lower_output.contains("task completed") || 
       lower_output.contains("implementation finished") ||
       lower_output.contains("successfully implemented") {
        return Some(CompletionResult {
            success: true,
            files_modified: extract_modified_files(output),
            summary: extract_summary(output),
        });
    }
    
    None
}

fn extract_modified_files(output: &str) -> Vec<String> {
    let mut files = Vec::new();
    
    // Look for file patterns in output
    let file_patterns = [
        r"(?:modified|created|updated|edited).*?([a-zA-Z0-9_./]+\.[a-zA-Z]+)",
        r"([a-zA-Z0-9_./]+\.[a-zA-Z]+).*?(?:has been|was).*?(?:modified|created|updated)",
        r"Writing.*?([a-zA-Z0-9_./]+\.[a-zA-Z]+)",
    ];
    
    for pattern in &file_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            for cap in re.captures_iter(output) {
                if let Some(file) = cap.get(1) {
                    files.push(file.as_str().to_string());
                }
            }
        }
    }
    
    files.sort();
    files.dedup();
    files
}

fn extract_summary(output: &str) -> String {
    // Extract the last meaningful paragraph as summary
    let lines: Vec<&str> = output.lines().collect();
    let mut summary_lines = Vec::new();
    
    for line in lines.iter().rev() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            summary_lines.push(trimmed);
            if summary_lines.len() >= 3 { // Take last 3 meaningful lines
                break;
            }
        }
    }
    
    summary_lines.reverse();
    summary_lines.join(" ")
}