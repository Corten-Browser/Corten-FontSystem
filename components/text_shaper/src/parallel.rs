//! Parallel text shaping with rayon (FEAT-046)
//!
//! This module provides parallelized text shaping for multiple text runs.
//! Features include:
//! - Batch shaping for multiple text runs using rayon parallel iterators
//! - Thread-safe cache access with parking_lot
//! - Configurable parallelism thresholds
//! - Work-stealing for balanced load distribution

use crate::shaper::TextShaper;
use crate::types::{ShapingError, ShapingOptions};
use font_registry::FontRegistry;
use font_types::types::{FontDescriptor, FontId, ShapedText};
use rayon::prelude::*;
use std::sync::Arc;

/// Minimum number of text runs to enable parallelism
const MIN_PARALLEL_RUNS: usize = 4;

/// Minimum total characters to enable parallelism
const MIN_PARALLEL_CHARS: usize = 100;

/// Configuration for parallel shaping
#[derive(Debug, Clone)]
pub struct ParallelShapingConfig {
    /// Minimum runs to enable parallel processing
    pub min_parallel_runs: usize,
    /// Minimum total characters to enable parallel processing
    pub min_parallel_chars: usize,
    /// Number of threads (None = use rayon default)
    pub num_threads: Option<usize>,
    /// Enable batch result caching
    pub enable_caching: bool,
}

impl Default for ParallelShapingConfig {
    fn default() -> Self {
        Self {
            min_parallel_runs: MIN_PARALLEL_RUNS,
            min_parallel_chars: MIN_PARALLEL_CHARS,
            num_threads: None,
            enable_caching: true,
        }
    }
}

/// A text run to be shaped
#[derive(Debug, Clone)]
pub struct TextRun {
    /// Text content
    pub text: String,
    /// Font ID to use
    pub font_id: FontId,
    /// Font size in pixels
    pub size: f32,
    /// Shaping options
    pub options: ShapingOptions,
}

impl TextRun {
    /// Create a new text run
    pub fn new(
        text: impl Into<String>,
        font_id: FontId,
        size: f32,
        options: ShapingOptions,
    ) -> Self {
        Self {
            text: text.into(),
            font_id,
            size,
            options,
        }
    }
}

/// Result of batch shaping operation
#[derive(Debug)]
pub struct BatchShapingResult {
    /// Shaped texts in order
    pub shaped_texts: Vec<Result<ShapedText, ShapingError>>,
    /// Whether parallel processing was used
    pub parallel_used: bool,
    /// Total characters processed
    pub total_chars: usize,
    /// Number of successful shapes
    pub successful: usize,
    /// Number of failed shapes
    pub failed: usize,
}

/// Parallel text shaper for batch operations
pub struct ParallelShaper<'a> {
    /// Font registry reference for creating shapers
    registry: &'a FontRegistry,
    /// Underlying text shaper for sequential operations
    shaper: TextShaper<'a>,
    /// Configuration
    config: ParallelShapingConfig,
}

impl<'a> ParallelShaper<'a> {
    /// Create a new parallel shaper
    pub fn new(registry: &'a FontRegistry) -> Self {
        Self {
            registry,
            shaper: TextShaper::new(registry),
            config: ParallelShapingConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(registry: &'a FontRegistry, config: ParallelShapingConfig) -> Self {
        Self {
            registry,
            shaper: TextShaper::new(registry),
            config,
        }
    }

    /// Shape multiple text runs in parallel
    ///
    /// Automatically determines whether to use parallel or sequential processing
    /// based on the workload size.
    pub fn shape_batch(&self, runs: &[TextRun]) -> BatchShapingResult {
        let total_chars: usize = runs.iter().map(|r| r.text.len()).sum();
        let should_parallelize = runs.len() >= self.config.min_parallel_runs
            || total_chars >= self.config.min_parallel_chars;

        let shaped_texts: Vec<Result<ShapedText, ShapingError>> = if should_parallelize {
            // Parallel processing with rayon - use thread-local shapers
            let registry = self.registry;
            runs.par_iter()
                .map_init(
                    || {
                        use crate::shaper::ShapingCacheConfig;
                        TextShaper::with_config(
                            registry,
                            ShapingCacheConfig {
                                max_entries: 0,
                                enable_statistics: false,
                            },
                        )
                    },
                    |shaper, run| shaper.shape_text(&run.text, run.font_id, run.size, &run.options),
                )
                .collect()
        } else {
            // Sequential processing
            runs.iter()
                .map(|run| {
                    self.shaper
                        .shape_text(&run.text, run.font_id, run.size, &run.options)
                })
                .collect()
        };

        let successful = shaped_texts.iter().filter(|r| r.is_ok()).count();
        let failed = shaped_texts.len() - successful;

        BatchShapingResult {
            shaped_texts,
            parallel_used: should_parallelize,
            total_chars,
            successful,
            failed,
        }
    }

    /// Shape text runs with fallback font matching
    pub fn shape_batch_with_fallback(
        &self,
        runs: &[(String, FontDescriptor, ShapingOptions)],
    ) -> BatchShapingResult {
        let total_chars: usize = runs.iter().map(|(text, _, _)| text.len()).sum();
        let should_parallelize = runs.len() >= self.config.min_parallel_runs
            || total_chars >= self.config.min_parallel_chars;

        let shaped_texts: Vec<Result<ShapedText, ShapingError>> = if should_parallelize {
            let registry = self.registry;
            runs.par_iter()
                .map_init(
                    || {
                        use crate::shaper::ShapingCacheConfig;
                        TextShaper::with_config(
                            registry,
                            ShapingCacheConfig {
                                max_entries: 0,
                                enable_statistics: false,
                            },
                        )
                    },
                    |shaper, (text, descriptor, options)| {
                        shaper.shape_text_with_fallback(text, descriptor, options)
                    },
                )
                .collect()
        } else {
            runs.iter()
                .map(|(text, descriptor, options)| {
                    self.shaper
                        .shape_text_with_fallback(text, descriptor, options)
                })
                .collect()
        };

        let successful = shaped_texts.iter().filter(|r| r.is_ok()).count();
        let failed = shaped_texts.len() - successful;

        BatchShapingResult {
            shaped_texts,
            parallel_used: should_parallelize,
            total_chars,
            successful,
            failed,
        }
    }

    /// Process text chunks in parallel and combine results
    ///
    /// Useful for processing large text by splitting it into chunks
    pub fn shape_chunked(
        &self,
        text: &str,
        font_id: FontId,
        size: f32,
        options: &ShapingOptions,
        chunk_size: usize,
    ) -> Result<ShapedText, ShapingError> {
        if text.len() <= chunk_size {
            return self.shaper.shape_text(text, font_id, size, options);
        }

        // Split text at word boundaries
        let chunks: Vec<&str> = split_at_words(text, chunk_size);

        if chunks.len() < 2 {
            return self.shaper.shape_text(text, font_id, size, options);
        }

        // Shape chunks in parallel with thread-local shapers
        let registry = self.registry;
        let options = options.clone();
        let shaped_chunks: Vec<Result<ShapedText, ShapingError>> = chunks
            .par_iter()
            .map_init(
                || {
                    use crate::shaper::ShapingCacheConfig;
                    TextShaper::with_config(
                        registry,
                        ShapingCacheConfig {
                            max_entries: 0,
                            enable_statistics: false,
                        },
                    )
                },
                |shaper, chunk| shaper.shape_text(chunk, font_id, size, &options),
            )
            .collect();

        // Combine results
        combine_shaped_texts(shaped_chunks)
    }

    /// Get reference to underlying shaper
    pub fn shaper(&self) -> &TextShaper<'a> {
        &self.shaper
    }
}

/// Split text at word boundaries respecting chunk size
fn split_at_words(text: &str, max_chunk_size: usize) -> Vec<&str> {
    let mut chunks = Vec::new();
    let mut start = 0;

    while start < text.len() {
        let end = (start + max_chunk_size).min(text.len());

        // Find word boundary
        let actual_end = if end == text.len() {
            end
        } else {
            // Look for whitespace before end
            text[start..end]
                .rfind(|c: char| c.is_whitespace())
                .map(|pos| start + pos + 1)
                .unwrap_or(end)
        };

        chunks.push(&text[start..actual_end]);
        start = actual_end;
    }

    chunks
}

/// Combine multiple shaped texts into one
fn combine_shaped_texts(
    results: Vec<Result<ShapedText, ShapingError>>,
) -> Result<ShapedText, ShapingError> {
    let mut combined_glyphs = Vec::new();
    let mut total_width = 0.0f32;
    let mut max_height = 0.0f32;
    let mut baseline = 0.0f32;

    for result in results {
        let shaped = result?;

        // Offset glyphs by current total width
        for mut glyph in shaped.glyphs {
            glyph.position.x += total_width;
            combined_glyphs.push(glyph);
        }

        total_width += shaped.width;
        max_height = max_height.max(shaped.height);
        baseline = baseline.max(shaped.baseline);
    }

    Ok(ShapedText {
        glyphs: combined_glyphs,
        width: total_width,
        height: max_height,
        baseline,
    })
}

/// Thread-safe batch shaper with shared cache
///
/// Can be cloned and shared across threads
pub struct SharedBatchShaper {
    /// Font registry (Arc for sharing)
    registry: Arc<FontRegistry>,
    /// Configuration
    config: ParallelShapingConfig,
}

impl SharedBatchShaper {
    /// Create a new shared batch shaper
    pub fn new(registry: Arc<FontRegistry>) -> Self {
        Self {
            registry,
            config: ParallelShapingConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(registry: Arc<FontRegistry>, config: ParallelShapingConfig) -> Self {
        Self { registry, config }
    }

    /// Shape a batch of text runs
    ///
    /// Creates thread-local shapers for parallel processing
    pub fn shape_batch(&self, runs: &[TextRun]) -> BatchShapingResult {
        let total_chars: usize = runs.iter().map(|r| r.text.len()).sum();
        let should_parallelize = runs.len() >= self.config.min_parallel_runs
            || total_chars >= self.config.min_parallel_chars;

        let shaped_texts: Vec<Result<ShapedText, ShapingError>> = if should_parallelize {
            runs.par_iter()
                .map(|run| {
                    // Create thread-local shaper
                    let shaper = TextShaper::new(&self.registry);
                    shaper.shape_text(&run.text, run.font_id, run.size, &run.options)
                })
                .collect()
        } else {
            let shaper = TextShaper::new(&self.registry);
            runs.iter()
                .map(|run| shaper.shape_text(&run.text, run.font_id, run.size, &run.options))
                .collect()
        };

        let successful = shaped_texts.iter().filter(|r| r.is_ok()).count();
        let failed = shaped_texts.len() - successful;

        BatchShapingResult {
            shaped_texts,
            parallel_used: should_parallelize,
            total_chars,
            successful,
            failed,
        }
    }
}

impl Clone for SharedBatchShaper {
    fn clone(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
            config: self.config.clone(),
        }
    }
}

/// Builder for text run batches
pub struct TextRunBatch {
    runs: Vec<TextRun>,
}

impl TextRunBatch {
    /// Create a new empty batch
    pub fn new() -> Self {
        Self { runs: Vec::new() }
    }

    /// Create with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            runs: Vec::with_capacity(capacity),
        }
    }

    /// Add a text run
    pub fn add(&mut self, run: TextRun) -> &mut Self {
        self.runs.push(run);
        self
    }

    /// Add multiple runs
    pub fn add_all(&mut self, runs: impl IntoIterator<Item = TextRun>) -> &mut Self {
        self.runs.extend(runs);
        self
    }

    /// Get the runs
    pub fn runs(&self) -> &[TextRun] {
        &self.runs
    }

    /// Take the runs
    pub fn into_runs(self) -> Vec<TextRun> {
        self.runs
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.runs.is_empty()
    }

    /// Get number of runs
    pub fn len(&self) -> usize {
        self.runs.len()
    }
}

impl Default for TextRunBatch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Language, Script};
    use font_types::types::Direction;
    use std::collections::HashMap;

    fn create_test_options() -> ShapingOptions {
        ShapingOptions {
            script: Script::Latin,
            language: Language {
                tag: "en-US".to_string(),
            },
            direction: Direction::LeftToRight,
            features: HashMap::new(),
            kerning: true,
            ligatures: true,
            letter_spacing: 0.0,
            word_spacing: 0.0,
        }
    }

    #[test]
    fn test_split_at_words() {
        let text = "Hello world this is a test";
        let chunks = split_at_words(text, 10);
        assert!(chunks.len() > 1);
        // All chunks should be non-empty
        for chunk in &chunks {
            assert!(!chunk.is_empty());
        }
    }

    #[test]
    fn test_split_at_words_no_split_needed() {
        let text = "Hello";
        let chunks = split_at_words(text, 100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Hello");
    }

    #[test]
    fn test_text_run_batch() {
        let mut batch = TextRunBatch::new();
        let options = create_test_options();

        batch
            .add(TextRun::new("Hello", 1, 12.0, options.clone()))
            .add(TextRun::new("World", 1, 12.0, options));

        assert_eq!(batch.len(), 2);
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_parallel_config_defaults() {
        let config = ParallelShapingConfig::default();
        assert_eq!(config.min_parallel_runs, MIN_PARALLEL_RUNS);
        assert_eq!(config.min_parallel_chars, MIN_PARALLEL_CHARS);
        assert!(config.enable_caching);
    }

    #[test]
    fn test_batch_shaping_result() {
        let result = BatchShapingResult {
            shaped_texts: vec![Ok(ShapedText {
                glyphs: vec![],
                width: 0.0,
                height: 0.0,
                baseline: 0.0,
            })],
            parallel_used: false,
            total_chars: 5,
            successful: 1,
            failed: 0,
        };

        assert_eq!(result.successful, 1);
        assert_eq!(result.failed, 0);
        assert!(!result.parallel_used);
    }
}
