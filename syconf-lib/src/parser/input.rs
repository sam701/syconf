use nom_locate::LocatedSpan;
use nom::IResult;

pub type Input<'a> = LocatedSpan<&'a str>;


// use std::ops::{Range, RangeTo};
//
// use nom::bytes::complete::tag;
// use nom::error::{ErrorKind, ParseError};
// use nom::lib::std::ops::{Index, RangeFrom};
// use nom::{
//     Compare, CompareResult, IResult, InputIter, InputLength, InputTake, InputTakeAtPosition,
//     Offset, Slice,
// };
//
// #[derive(Debug, Eq, PartialEq, Clone)]
// pub struct Input<'a> {
//     content: &'a str,
//     position: usize,
// }
//
// impl<'a> Input<'a> {
//     pub fn new(content: &'a str, position: usize) -> Self {
//         Self { content, position }
//     }
//
//     pub fn as_str(&self) -> &str {
//         self.content
//     }
//
//     pub fn position(&self) -> usize {
//         self.position
//     }
//
//     fn adjust_split(&self, (rest, result): (&str, &str)) -> (Input, Input) {
//         (
//             Input::new(rest, self.position + result.len()),
//             Input::new(result, self.position),
//         )
//     }
// }
//
// impl<'a> From<&str> for Input<'a> {
//     fn from(content: &str) -> Self {
//         Input {
//             content,
//             position: 0,
//         }
//     }
// }
//
// impl<'a> InputTake for Input<'a> {
//     fn take(&self, count: usize) -> Self {
//         if count > self.content.len() {
//             panic!("Count > content length");
//         }
//         Input::new(&self.content[..count], self.position)
//     }
//
//     fn take_split(&self, count: usize) -> (Self, Self) {
//         if count > self.content.len() {
//             panic!("Count > content length");
//         }
//         (
//             Input::new(&self.content[count..], self.position + count),
//             Input::new(&self.content[..count], self.position),
//         )
//     }
// }
//
// impl<'a> Compare<&str> for Input<'a> {
//     fn compare(&self, t: &str) -> CompareResult {
//         self.content.compare(t)
//     }
//
//     fn compare_no_case(&self, t: &str) -> CompareResult {
//         self.content.compare_no_case(t)
//     }
// }
//
// #[test]
// fn input() {
//     let txt = Input::new("abcdef", 5);
//     let result: IResult<Input, Input> = tag("abc")(txt);
//     assert_eq!(result, Ok((Input::new("def", 8), Input::new("abc", 5),)));
// }
//
//
// pub fn position(s: Input) -> IResult<Input, usize> {
//     let p = s.location_offset();
//     Ok((s, p))
// }
//
//
// impl<'a> InputLength for Input<'a> {
//     fn input_len(&self) -> usize {
//         self.content.len()
//     }
// }
//
// impl<'a> InputTakeAtPosition for Input<'a> {
//     type Item = char;
//
//     fn split_at_position<P, E: ParseError<Self>>(&self, predicate: P) -> IResult<Self, Self, E>
//     where
//         P: Fn(Self::Item) -> bool,
//     {
//         self.content
//             .split_at_position(predicate)
//             .map(self.adjust_split)
//     }
//
//     fn split_at_position1<P, E: ParseError<Self>>(
//         &self,
//         predicate: P,
//         e: ErrorKind,
//     ) -> IResult<Self, Self, E>
//     where
//         P: Fn(Self::Item) -> bool,
//     {
//         self.content
//             .split_at_position1(predicate, e)
//             .map(self.adjust_split)
//     }
//
//     fn split_at_position_complete<P, E: ParseError<Self>>(
//         &self,
//         predicate: P,
//     ) -> IResult<Self, Self, E>
//     where
//         P: Fn(Self::Item) -> bool,
//     {
//         self.content
//             .split_at_position_complete(predicate)
//             .map(self.adjust_split)
//     }
//
//     fn split_at_position1_complete<P, E: ParseError<Self>>(
//         &self,
//         predicate: P,
//         e: ErrorKind,
//     ) -> IResult<Self, Self, E>
//     where
//         P: Fn(Self::Item) -> bool,
//     {
//         self.content
//             .split_at_position_complete1(predicate)
//             .map(self.adjust_split)
//     }
// }
//
// impl<'a> Offset for Input<'a> {
//     fn offset(&self, second: &Self) -> usize {
//         self.content.offset(second.content)
//     }
// }
//
// impl<'a> Slice<RangeTo<usize>> for Input<'a> {
//     fn slice(&self, range: RangeTo<usize>) -> Self {
//         Input {
//             content: &self.content[..range.end],
//             position: self.position,
//         }
//     }
// }
//
// impl<'a> Slice<RangeFrom<usize>> for Input<'a> {
//     fn slice(&self, range: RangeFrom<usize>) -> Self {
//         Input {
//             content: &self.content[range.start..],
//             position: self.position + range.start,
//         }
//     }
// }
//
// impl<'a> Slice<Range<usize>> for Input<'a> {
//     fn slice(&self, range: Range<usize>) -> Self {
//         Input {
//             content: &self.content[range.start..range.end],
//             position: self.position + range.start,
//         }
//     }
// }
