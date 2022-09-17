//! Replacements for nom functions missing without the `alloc` feature

use core::fmt::Debug;

use crate::lib;
use nom::{
    error::{ErrorKind, ParseError},
    IResult, InputLength, Parser,
};

pub fn many_m_n<I, O, E, F, const MAX: usize>(
    min: usize,
    mut parse: F,
) -> impl FnMut(I) -> IResult<I, lib::std::vec::Vec<O, MAX>, E>
where
    I: Clone + InputLength,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    move |mut input: I| {
        if min > MAX {
            return Err(nom::Err::Failure(E::from_error_kind(
                input,
                ErrorKind::ManyMN,
            )));
        }

        let mut res = crate::lib::std::vec::Vec::new();
        for count in 0..MAX {
            let len = input.input_len();
            match parse.parse(input.clone()) {
                Ok((tail, value)) => {
                    // infinite loop check: the parser must always consume
                    if tail.input_len() == len {
                        return Err(nom::Err::Error(E::from_error_kind(
                            input,
                            ErrorKind::ManyMN,
                        )));
                    }

                    // loop is limited by the same const generic as the vector size,
                    // so `res` will not be full
                    unsafe { res.push_unchecked(value) };
                    input = tail;
                }
                Err(nom::Err::Error(e)) => {
                    if count < min {
                        return Err(nom::Err::Error(E::append(input, ErrorKind::ManyMN, e)));
                    } else {
                        return Ok((input, res));
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok((input, res))
    }
}

pub fn count<I, O: Debug, E, F, const VEC_SIZE: usize>(
    mut f: F,
    count: usize,
) -> impl FnMut(I) -> IResult<I, lib::std::vec::Vec<O, VEC_SIZE>, E>
where
    I: Clone + PartialEq,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    debug_assert!(count <= VEC_SIZE);
    move |i: I| {
        let mut input = i.clone();
        let mut res = crate::lib::std::vec::Vec::new();

        for _ in 0..count {
            let input_ = input.clone();
            match f.parse(input_) {
                Ok((i, o)) => {
                    res.push(o).expect("Pushing item to full Vec");
                    input = i;
                }
                Err(nom::Err::Error(e)) => {
                    return Err(nom::Err::Error(E::append(i, ErrorKind::Count, e)));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok((input, res))
    }
}
