use crate::generator::GeneratorColumn;
use crate::random::RandomNumberStream;
use std::marker::PhantomData;

/// Abstract base for row generators (AbstractRowGenerator)
///
/// Manages the random number streams for the generator columns `C` of one
/// table and walks the source rows `current_row..=row_count` the generator
/// is to produce.
pub struct AbstractRowGenerator<C> {
    /// The next source row to generate (1-based)
    current_row: u64,
    /// The last source row to generate (inclusive)
    row_count: u64,
    /// One stream per column in `C::values()`, in that order
    random_number_streams: Vec<RandomNumberStream>,
    /// Global column number of the first column, so a column's stream is at
    /// index `global_column_number - base_global_column_number`
    base_global_column_number: i32,
    column: PhantomData<C>,
}

impl<C: GeneratorColumn> AbstractRowGenerator<C> {
    /// Create a new abstract row generator for source rows `1..=row_count`.
    ///
    /// Pre-creates the random number streams of all generator columns
    /// (matching Java's AbstractRowGenerator constructor behavior)
    pub fn new(row_count: u64) -> Self {
        let columns = C::values();
        let base_global_column_number = columns
            .first()
            .map(|column| column.get_global_column_number())
            .unwrap_or(0);

        // Pre-create all streams for this table's generator columns
        // This is critical because consume_remaining_seeds_for_row needs to advance
        // ALL streams, even ones that haven't been accessed yet
        let random_number_streams = columns
            .iter()
            .enumerate()
            .map(|(index, column)| {
                let global_column_number = column.get_global_column_number();
                // get_random_number_stream relies on contiguous column numbers
                assert_eq!(
                    global_column_number,
                    base_global_column_number + index as i32,
                    "generator columns must have contiguous global column numbers"
                );
                RandomNumberStream::new_with_column(
                    global_column_number,
                    column.get_seeds_per_row(),
                )
            })
            .collect();

        Self {
            current_row: 1,
            row_count,
            random_number_streams,
            base_global_column_number,
            column: PhantomData,
        }
    }

    /// Get the random number stream for a generator column
    #[inline]
    pub fn get_random_number_stream(&mut self, column: &C) -> &mut RandomNumberStream {
        let index = (column.get_global_column_number() - self.base_global_column_number) as usize;
        &mut self.random_number_streams[index]
    }

    /// Consume remaining seeds for all streams (AbstractRowGenerator.consumeRemainingSeedsForRow)
    pub fn consume_remaining_seeds_for_row(&mut self) {
        use crate::random::RandomValueGenerator;

        for stream in self.random_number_streams.iter_mut() {
            // Consume remaining seeds until each stream has used its full seeds_per_row allocation
            while stream.get_seeds_used() < stream.get_seeds_per_row() {
                RandomValueGenerator::generate_uniform_random_int(1, 100, stream);
            }
            // Reset seeds used count for next row
            stream.reset_seeds_used();
        }
    }

    /// Start generating at `starting_row_number` (1-based), fast forwarding
    /// all streams to that row.
    pub fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        let rows_to_skip = starting_row_number.saturating_sub(1);
        for stream in self.random_number_streams.iter_mut() {
            stream.skip_rows(rows_to_skip);
        }
        self.current_row = starting_row_number;
    }

    /// Restrict generation to source rows
    /// `starting_row_number..=ending_row_number` (1-based, inclusive).
    ///
    /// The ending row number is clamped to the row count.
    pub fn set_source_row_range(&mut self, starting_row_number: u64, ending_row_number: u64) {
        self.skip_rows_until_starting_row_number(starting_row_number);
        self.row_count = self.row_count.min(ending_row_number);
    }

    /// The source row to generate next, or `None` when past the end of the
    /// range.
    ///
    /// Call [`Self::finish_row`] once the row is generated to move on.
    pub fn next_row_number(&self) -> Option<u64> {
        (self.current_row <= self.row_count).then_some(self.current_row)
    }

    /// Consume the seeds the current row left unused and move to the next
    /// row.
    pub fn finish_row(&mut self) {
        self.consume_remaining_seeds_for_row();
        self.current_row += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::CallCenterGeneratorColumn;

    #[test]
    fn creates_one_stream_per_column() {
        let mut generator = AbstractRowGenerator::<CallCenterGeneratorColumn>::new(6);
        assert_eq!(generator.random_number_streams.len(), 34);

        let stream = generator.get_random_number_stream(&CallCenterGeneratorColumn::CcNulls);
        assert_eq!(stream.get_seeds_per_row(), 2);
    }

    #[test]
    fn walks_the_source_row_range() {
        let mut generator = AbstractRowGenerator::<CallCenterGeneratorColumn>::new(6);
        assert_eq!(generator.next_row_number(), Some(1));
        generator.finish_row();
        assert_eq!(generator.next_row_number(), Some(2));

        generator.set_source_row_range(5, 100);
        assert_eq!(generator.next_row_number(), Some(5));
        generator.finish_row();
        assert_eq!(generator.next_row_number(), Some(6));
        generator.finish_row();
        assert_eq!(generator.next_row_number(), None);

        generator.skip_rows_until_starting_row_number(3);
        assert_eq!(generator.next_row_number(), Some(3));
    }
}
