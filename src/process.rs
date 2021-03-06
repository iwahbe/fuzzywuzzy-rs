//! Convenience methods to process fuzzy matching queries for common use cases.

/// Score multiple options against a base query string and return all exceeding a cutoff.
///
/// Returns a Vec with the options and their match score if their score is above the cutoff.
/// Results are configurable using custom text processors and scorers.
/// Good default choices are `utils::full_process` as the processor, `fuzz:wratio` as the scorer, and zero as the score_cutoff.
///
/// ```
/// # use fuzzywuzzy::process::extract_without_order;
/// # use fuzzywuzzy::fuzz::wratio;
/// let choices = vec![
///     "new york mets vs chicago cubs",
///     "chicago cubs vs chicago white sox",
///     "philladelphia phillies vs atlanta braves",
///     "braves vs mets",
/// ];
/// let expected_results = vec![
///    ("new york mets vs chicago cubs".to_string(), 86u8),
///    ("chicago cubs vs chicago white sox".to_string(), 86u8),
///    ("philladelphia phillies vs atlanta braves".to_string(), 54u8),
///    ("braves vs mets".to_string(), 57u8)
/// ];
/// assert_eq!(
///     extract_without_order(
///         "brave new cubs",
///         choices,
///         |s, b| s.into(), // an alternative to full_process.
///         &wratio,
///         0),
///     expected_results);
/// ```
pub fn extract_without_order<I, T, P, S>(
    query: &str,
    choices: I,
    processor: P,
    scorer: S,
    score_cutoff: u8,
) -> Vec<(String, u8)>
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
    P: Fn(&str, bool) -> String,
    S: Fn(&str, &str, bool, bool) -> u8,
{
    let processed_query: String = processor(query, false);
    if processed_query.is_empty() {
        // TODO: Make warning configurable, instead of being printed by default.
        // println!("Applied processor reduces input query to empty string, all comparisons will have score 0. [Query: '{0}']", processed_query.as_str());
    }

    // See: https://github.com/logannc/fuzzyrusty/issues/6
    // TODO: Check if scorer in list of known processor functions to avoid calling utils::full_process multiple times.
    // TODO: Only process the query once instead of for every choice.

    let mut results = vec![];
    for choice in choices {
        let processed: String = processor(choice.as_ref(), false);
        let score: u8 = scorer(processed_query.as_str(), processed.as_str(), true, true);
        if score >= score_cutoff {
            results.push((choice.as_ref().to_string(), score))
        }
    }
    results
}

/// Score multiple options against a base query string and return the best one exceeding a cutoff.
///
/// This is a convenience method which returns the single best choice from `extract_without_order`.
///
/// For compatibility with `fuzzywuzzy-py`, if there is a tie for the best choice, the first one is returned.
/// (This is the opposite of how `Iterator::max_by` works.)
///
/// ```
/// # use fuzzywuzzy::process::extract_one;
/// use fuzzywuzzy::fuzz::wratio;
/// use fuzzywuzzy::utils::full_process;
/// let choices = vec![
///     "new york mets vs chicago cubs",
///     "chicago cubs vs chicago white sox",
///     "philladelphia phillies vs atlanta braves",
///     "braves vs mets",
/// ];
/// assert_eq!(
///    extract_one("brave new cubs",
///       choices.iter(),
///       &full_process,
///       &wratio,
///       0).unwrap().0,
///    choices[0]
/// );
/// assert_eq!(
///    extract_one(
///       "new york mets at atlanta braves",
///       choices.iter(),
///       &full_process,
///       &wratio,
///       0).unwrap().0,
///    choices[3]
/// );
/// assert_eq!(
///    extract_one(
///       "philadelphia phillies at atlanta braves",
///       choices.iter(),
///       &full_process,
///       &wratio,
///       0).unwrap().0,
///    choices[2]
/// );
/// assert_eq!(
///    extract_one(
///       "atlanta braves at philadelphia phillies",
///       choices.iter(),
///       &full_process,
///       &wratio,
///       0).unwrap().0,
///    choices[2]
/// );
/// assert_eq!(
///    extract_one(
///       "chicago cubs vs new york mets",
///       choices.iter(),
///       &full_process,
///       &wratio,
///       0).unwrap().0,
///    choices[0]
/// );
/// ```
pub fn extract_one<I, T, P, S>(
    query: &str,
    choices: I,
    processor: P,
    scorer: S,
    score_cutoff: u8,
) -> Option<(String, u8)>
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
    P: Fn(&str, bool) -> String,
    S: Fn(&str, &str, bool, bool) -> u8,
{
    let best = extract_without_order(query, choices, processor, scorer, score_cutoff);
    if best.is_empty() {
        return None;
    }
    best.iter()
        // Python and Rust have different semantics for which maximum value from an iterator is
        // returned when there are multiple equal values! In Python (when using max built-in), if
        // multiple items are maximal, the function returns the first one encountered. In Rust (when
        // using .max_by), if several elements are equally maximum, the last element is returned.
        //
        // The solution here is to reverse the iterator to ensure the actual first item in the
        // original ordering of `choices` is returned (as this is the behavior of fuzzywuzzy).
        .rev()
        .cloned()
        .max_by(|(_, acc_score), (_, score)| acc_score.cmp(score))
}
