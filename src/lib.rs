use std::iter::{Iterator, Peekable};


/// Returns the closest match from the given options to the given query.
///
/// This algorithm works by scanning through each option trying to match the beginning of the
/// query. Once a match has begun, any non-matching characters will cause the scan to skip to the
/// next word of the option. If the end of the option is reached before the entire query has been
/// matched somewhere, the option is considered not to match.
///
/// If multiple options match, it returns the shortest.
///
/// # Examples
///
/// ```
/// let options = &["one two", "three four", "five six"];
/// let query = "owo";
/// let result = close_enough::close_enough(options, query);
/// assert_eq!(result, Some(&"one two"));
/// ```
pub fn close_enough<I, O, Q>(options: I, query: Q) -> Option<O>
    where I: IntoIterator<Item=O>, O: AsRef<str>, Q: AsRef<str>
{
    let mut shortest_answer: Option<O> = None;

    for opt in options
    {
        let matches = {
            let mut optchars = opt.as_ref().chars().peekable();
            let mut querychars = query.as_ref().chars().peekable();

            while querychars.peek().is_some()
            {
                while optchars.peek().is_some() && !same_char(querychars.peek(), optchars.peek())
                {
                    optchars.next();
                }

                if optchars.peek().is_none() { break; }

                while querychars.peek().is_some() && same_char(querychars.peek(), optchars.peek())
                {
                    querychars.next();
                    optchars.next();
                }

                skip_word(&mut optchars);
            }

            querychars.peek().is_none()
        };

        if matches
        {
            shortest_answer = Some(select_shortest(opt, shortest_answer));
        }
    }

    shortest_answer
}


fn same_char(a: Option<&char>, b: Option<&char>) -> bool
{
    match (a, b)
    {
        (Some(achar), Some(bchar)) => achar.to_lowercase().next() == bchar.to_lowercase().next(),
        _ => false
    }
}


fn select_shortest<T>(proposed: T, previous: Option<T>) -> T
    where T: AsRef<str>
{
    match previous
    {
        None => proposed,
        Some(prev) => if proposed.as_ref().len() < prev.as_ref().len() { proposed } else { prev }
    }
}


fn skip_word<I>(chars: &mut Peekable<I>)
    where I: Iterator<Item=char>
{
    let mut has_more = chars.peek().is_some();
    while has_more
    {
        if let Some(c) = chars.peek()
        {
            if !c.is_alphanumeric() || c.is_uppercase()
            {
                break;
            }
        }
        has_more = chars.next().is_some();
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    fn test(options: &[&str], query: &str, expected: Option<&str>)
    {
        assert_eq!(close_enough(options, query), expected.as_ref());
    }

    #[test]
    fn no_options_returns_none()
    {
        test(&[], "blah", None);
    }

    #[test]
    fn single_option_returned_on_match()
    {
        test(&["only"], "only", Some("only"));
    }

    #[test]
    fn single_option_returns_none_if_not_match()
    {
        test(&["only"], "different", None);
    }

    #[test]
    fn multiple_options_returns_matching()
    {
        test(&["one", "two"], "two", Some("two"));
    }

    #[test]
    fn partial_match_returns_answer()
    {
        test(&["only"], "on", Some("only"));
    }

    #[test]
    fn matching_is_case_insensitive()
    {
        test(&["OnLy"], "only", Some("OnLy"));
    }

    #[test]
    fn multiple_partial_matches_return_shortest_answer()
    {
        test(&["item_the_first", "item"], "it", Some("item"));
    }

    #[test]
    fn entire_query_must_be_present_for_match()
    {
        test(&["item_the_first"], "item_the_fist", None);
    }

    #[test]
    fn can_match_from_beyond_start()
    {
        test(&["theonlyitem"], "only", Some("theonlyitem"));
    }

    #[test]
    fn failed_match_looks_to_next_word()
    {
        test(&["A very_big-longMatch"], "avblm", Some("A very_big-longMatch"));
    }

    #[test]
    fn failed_match_does_not_look_within_same_word()
    {
        test(&["averybiglongmatch"], "avblm", None);
    }

    #[test]
    fn works_on_useful_collection_types()
    {
        assert_eq!(close_enough(["a", "thing"].iter(), "thing"), Some(&"thing"));
        assert_eq!(close_enough(&["a", "thing"], "thing"), Some(&"thing"));
        assert_eq!(close_enough(&vec!["a", "thing"], "thing"), Some(&"thing"));
        assert_eq!(close_enough(&vec!["a".to_owned(), "thing".to_owned()], "thing"), Some(&"thing".to_owned()));
    }
}

