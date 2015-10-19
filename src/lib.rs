#![feature(zero_one)]

/// Kuhn-Munkres Algorithm (also called Hungarian algorithm) for solving the 
/// Assignment Problem.
///
/// Copyright (c) 2015 by Michael Neumann (mneumann@ntecs.de).
///
/// This code is derived from a port of the Python version found here:
/// https://github.com/bmc/munkres/blob/master/munkres.py
/// which is Copyright (c) 2008 Brian M. Clapper.


// TODO:
//    * Implement SquareMatrix. Get rid of nalgebra.
//    * Reuse path Vec in step5
//    * Cleanup

extern crate nalgebra as na;
extern crate bit_vec;

use na::{DMat, BaseNum};
use std::ops::{Neg, Sub};
use std::num::Zero;
use std::cmp;
use square_matrix::SquareMatrix;
use coverage::Coverage;

mod square_matrix;
mod coverage;

#[derive(Debug)]
pub struct WeightMatrix<T: Copy> {
    c: SquareMatrix<T>
}

impl<T> WeightMatrix<T> where T: BaseNum + Ord + Eq + Sub<Output=T> + Copy {
    pub fn from_row_vec(n: usize, data: Vec<T>) -> WeightMatrix<T> {
        WeightMatrix{c: SquareMatrix::from_row_vec(n, data)}
    }

    #[inline(always)]
    fn n(&self) -> usize { self.c.n() }

    fn is_element_zero(&self, pos: (usize, usize)) -> bool {
        self.c[pos] == T::zero()
    }

    /// Return the minimum element of row `row`.
    fn min_of_row(&self, row: usize) -> T {
        let mut min = self.c[(row, 0)];
        for col in 1 .. self.n() {
            min = cmp::min(min, self.c[(row, col)]);
        }
        min
    }  

    /// Return the minimum element of column `col`.
    fn min_of_col(&self, col: usize) -> T {
        let mut min = self.c[(0, col)];
        for row in 1 .. self.n() {
            min = cmp::min(min, self.c[(row, col)]);
        }
        min
    }

    // Subtract `val` from every element in row `row`. 
    fn sub_row(&mut self, row: usize, val: T) {
        for col in 0 .. self.n() {
            self.c[(row, col)] = self.c[(row, col)] - val;
        }
    }

    // Subtract `val` from every element in column `col`. 
    fn sub_col(&mut self, col: usize, val: T) {
        for row in 0 .. self.n() {
            self.c[(row, col)] = self.c[(row, col)] - val;
        }
    }

    // Add `val` to every element in row `row`. 
    fn add_row(&mut self, row: usize, val: T) {
        for col in 0 .. self.n() {
            self.c[(row, col)] = self.c[(row, col)] - val;
        }
    }

    /// Find the first uncovered element with value 0 `find_a_zero`
    fn find_uncovered_zero(&self, cov: &Coverage) -> Option<(usize, usize)> {
        let n = self.n();
        // XXX: Refactor into iterator

        for col in 0 .. n {
            if cov.is_col_covered(col) { continue; }
            for row in 0..n {
                if !cov.is_row_covered(row) && self.is_element_zero((row, col)) {
                    return Some((row, col));
                }
            }
        }
        return None;
    }

    /// Find the smallest uncovered value in the matrix
    fn find_uncovered_min(&self, cov: &Coverage) -> Option<T> {
        let mut min = None;
        let n = self.n();
        // XXX: Refactor into iterator
        for row in 0 .. n {
            if cov.is_row_covered(row) { continue; }
            for col in 0..n {
                if cov.is_col_covered(col) { continue; }
                let elm = self.c[(row, col)]; 
                min = Some(match min {
                    None => elm,
                    Some(m) => cmp::min(m, elm),
                });
            }
        }
        return min;
    }
}


#[derive(Debug, Eq, PartialEq)]
enum Step {
    Step1,
    Step2,
    Step3,
    Step4(Option<usize>),
    Step5(usize, usize),
    Step6,
    Done,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mark {
   None,
   Star,
   Prime
}

#[derive(Debug)]
struct MarkMatrix {
    n: usize,
    marks: DMat<Mark>
}

impl MarkMatrix {
    fn new(n: usize) -> MarkMatrix {
        MarkMatrix {n: n,
                    marks: DMat::from_elem(n, n, Mark::None)}
    }

    fn n(&self) -> usize { self.n }

    fn toggle_star(&mut self, pos: (usize, usize)) {
        if self.is_star(pos) {
            self.unmark(pos);
        } else {
            self.star(pos);
        }
    }

    fn get(&self, pos: (usize, usize)) -> Mark {
        self.marks[pos]
    }

    fn unmark(&mut self, pos: (usize, usize)) {
       self.marks[pos] = Mark::None; 
    }

    fn star(&mut self, pos: (usize, usize)) {
       self.marks[pos] = Mark::Star; 
    }

    fn prime(&mut self, pos: (usize, usize)) {
       self.marks[pos] = Mark::Prime; 
    }

    fn is_star(&self, pos: (usize, usize)) -> bool {
        match self.marks[pos] {
            Mark::Star => true,
            _          => false 
        }
    }

    fn is_prime(&self, pos: (usize, usize)) -> bool {
        match self.marks[pos] {
            Mark::Prime => true,
            _          => false 
        }
    }

    fn find_first_star_in_row(&self, row: usize) -> Option<usize> {
        for col in 0..self.n {
            if self.is_star((row, col)) {
                return Some(col);
            }
        }
        return None;
    }

    fn find_first_prime_in_row(&self, row: usize) -> Option<usize> {
        for col in 0..self.n {
            if self.is_prime((row, col)) {
                return Some(col);
            }
        }
        return None;
    }

    fn find_first_star_in_col(&self, col: usize) -> Option<usize> {
        for row in 0..self.n {
            if self.is_star((row, col)) {
                return Some(row);
            }
        }
        return None;
    }

    fn clear_primes(&mut self) {
        for row in 0..self.n {
            for col in 0..self.n {
                if let Mark::Prime = self.marks[(row, col)] {
                    self.marks[(row, col)] = Mark::None;
                }
            }
        }
    }
}

/// For each row of the matrix, find the smallest element and
/// subtract it from every element in its row. Go to Step 2.
fn step1<T>(c: &mut WeightMatrix<T>) -> Step 
where T: BaseNum + Ord {
    let n = c.n();

    for row in 0..n {
        let min = c.min_of_row(row);
        c.sub_row(row, min);
    }

    return Step::Step2;
}


/// Find a zero (Z) in the resulting matrix. If there is no starred
/// zero in its row or column, star Z. Repeat for each element in the
/// matrix. Go to Step 3.
fn step2<T>(c: &WeightMatrix<T>, marks: &mut MarkMatrix, cov: &mut Coverage) -> Step 
where T: BaseNum + Ord + Neg<Output=T> + Eq {
    let n = c.n();
    
    assert!(marks.n() == n);
    assert!(cov.n() == n);

    for row in 0 .. n {
        if cov.is_row_covered(row) { continue; }
        for col in 0..n {
            if cov.is_col_covered(col) { continue; }
            if c.is_element_zero((row, col)) {
                marks.star((row, col));
                cov.cover((row, col));
            }
        }
    }

    // clear covers
    cov.clear();

    return Step::Step3;
}

/// Cover each column containing a starred zero. If K columns are
/// covered, the starred zeros describe a complete set of unique
/// assignments. In this case, Go to DONE, otherwise, Go to Step 4.
fn step3<T>(c: &WeightMatrix<T>, marks: &MarkMatrix, cov: &mut Coverage) -> Step 
where T: BaseNum + Ord + Neg<Output=T> + Eq {
    let n = c.n();
    
    assert!(marks.n() == n);
    assert!(cov.n() == n);

    let mut count: usize = 0;

    for row in 0..n {
        for col in 0..n {
            if marks.is_star((row, col)) {
                cov.cover_col(col);
                count += 1;
            }
        }
    }

    if count >= n {
        Step::Done
    } else {
        Step::Step4(Some(count))
    }
}

/// Find a noncovered zero and prime it. If there is no starred zero
/// in the row containing this primed zero, Go to Step 5. Otherwise,
/// cover this row and uncover the column containing the starred
/// zero. Continue in this manner until there are no uncovered zeros
/// left. Save the smallest uncovered value and Go to Step 6.
fn step4<T>(c: &WeightMatrix<T>, marks: &mut MarkMatrix, cov: &mut Coverage) -> Step
where T: BaseNum + Ord + Neg<Output=T> + Eq {

    let n = c.n();

    assert!(marks.n() == n);
    assert!(cov.n() == n);

    loop {
        match c.find_uncovered_zero(cov) {
            None => {
                return Step::Step6;
            }
            Some((row, col)) => {
                marks.prime((row, col));
                match marks.find_first_star_in_row(row) {
                    Some(star_col) => {
                        cov.cover_row(row);
                        cov.uncover_col(star_col);
                    }
                    None => {
                        // in Python: self.Z0_r, self.Z0_c
                        return Step::Step5(row, col)
                    }
                }
            }
        }
    }
    
}

/// Construct a series of alternating primed and starred zeros as
/// follows. Let Z0 represent the uncovered primed zero found in Step 4.
/// Let Z1 denote the starred zero in the column of Z0 (if any).
/// Let Z2 denote the primed zero in the row of Z1 (there will always
/// be one). Continue until the series terminates at a primed zero
/// that has no starred zero in its column. Unstar each starred zero
/// of the series, star each primed zero of the series, erase all
/// primes and uncover every line in the matrix. Return to Step 3
fn step5<T>(c: &WeightMatrix<T>, marks: &mut MarkMatrix, cov: &mut Coverage, z0: (usize, usize)) -> Step
where T: BaseNum + Ord + Neg<Output=T> + Eq {

    let n = c.n();

    assert!(marks.n() == n);
    assert!(cov.n() == n);

    let mut path: Vec<(usize, usize)> = Vec::new();

    path.push(z0);

    loop {
        let &(prev_row, prev_col) = path.last().unwrap();

        match marks.find_first_star_in_col(prev_col) {
            Some(row) => {
                path.push((row, prev_col));

                if let Some(col) = marks.find_first_prime_in_row(row) {
                    path.push((row, col));
                } else {
                    // XXX
                    panic!("no prime in row");
                }
            }
            None => {
                break;
            }
        }
    }

    // convert_path
    for pos in path {
        marks.toggle_star(pos);
    }
    
    cov.clear();
    marks.clear_primes();
    return Step::Step3;
}


/// Add the value found in Step 4 to every element of each covered
/// row, and subtract it from every element of each uncovered column.
/// Return to Step 4 without altering any stars, primes, or covered
/// lines.
fn step6<T>(c: &mut WeightMatrix<T>, cov: &Coverage) -> Step
where T: BaseNum + Ord + Neg<Output=T> + Eq + Copy + Neg<Output=T> { 

    let n = c.n();
    assert!(cov.n() == n);

    let minval = c.find_uncovered_min(cov).unwrap(); 
    for row in 0..n {
        if cov.is_row_covered(row) {
            c.add_row(row, minval);
        }
    }
    for col in 0..n {
        if !cov.is_col_covered(col) {
            c.sub_col(col, minval);
        }
    }

    return Step::Step4(None);
}

pub fn solve_assignment<T>(weights: &mut WeightMatrix<T>) -> Vec<(usize,usize)>
where T: BaseNum + Ord + Neg<Output=T> + Eq + Copy {
   let n = weights.n();

   let mut marks = MarkMatrix::new(n);
   let mut coverage = Coverage::new(n);

   let mut step = Step::Step1;
   loop {
       match step {
            Step::Step1 => {
                step = step1(weights)
            }
            Step::Step2 => {
                step = step2(weights, &mut marks, &mut coverage);
            }
            Step::Step3 => {
                step = step3(weights, &marks, &mut coverage);
            }
            Step::Step4(_) => {
                step = step4(weights, &mut marks, &mut coverage);
            }
            Step::Step5(z0_r, z0_c) => {
                step = step5(weights, &mut marks, &mut coverage, (z0_r, z0_c));
            }
            Step::Step6 => {
                step = step6(weights, &coverage);
            }
            Step::Done => {
                break;
            }
       }
   }

   // now look for the starred elements
   let mut matching = Vec::with_capacity(n);
   for row in 0..n {
       for col in 0..n {
           if marks.is_star((row, col)) {
               matching.push((row,col));
           }
       }
   }
   assert!(matching.len() == n);

   return matching;
}

#[test]
fn test_step1() {
    let c = vec![250, 400, 350,
                 400, 600, 350,
                 200, 400, 250];

    let mut weights: WeightMatrix<i32> = WeightMatrix::from_row_vec(3, c);

    let next_step = step1(&mut weights);
    assert_eq!(Step::Step2, next_step);

    let exp = vec![0, 150, 100,
                   50, 250, 0,
                   0, 200, 50];

    assert_eq!(exp, weights.c.into_vec());
}


#[test]
fn test_step2() {
    let c = vec![0, 150, 100,
                 50, 250, 0,
                 0, 200, 50];

    let weights: WeightMatrix<i32> = WeightMatrix::from_row_vec(3, c);
    let mut marks = MarkMatrix::new(weights.n());
    let mut coverage = Coverage::new(weights.n());

    let next_step = step2(&weights, &mut marks, &mut coverage);
    assert_eq!(Step::Step3, next_step);

    assert_eq!(true, marks.is_star((0,0)));
    assert_eq!(false, marks.is_star((0,1)));
    assert_eq!(false, marks.is_star((0,2)));

    assert_eq!(false, marks.is_star((1,0)));
    assert_eq!(false, marks.is_star((1,1)));
    assert_eq!(true, marks.is_star((1,2)));
       
    assert_eq!(false, marks.is_star((2,0)));
    assert_eq!(false, marks.is_star((2,1)));
    assert_eq!(false, marks.is_star((2,2)));

    // coverage was cleared
    assert_eq!(false, coverage.is_row_covered(0));
    assert_eq!(false, coverage.is_row_covered(1));
    assert_eq!(false, coverage.is_row_covered(2));
    assert_eq!(false, coverage.is_col_covered(0));
    assert_eq!(false, coverage.is_col_covered(1));
    assert_eq!(false, coverage.is_col_covered(2));

/*
    assert_eq!(true, coverage.is_row_covered(0));
    assert_eq!(true, coverage.is_row_covered(1));
    assert_eq!(false, coverage.is_row_covered(2));

    assert_eq!(true, coverage.is_col_covered(0));
    assert_eq!(false, coverage.is_col_covered(1));
    assert_eq!(true, coverage.is_col_covered(2));
*/
}

#[test]
fn test_step3() {
    let c = vec![0, 150, 100,
                 50, 250, 0,
                 0, 200, 50];

    let weights: WeightMatrix<i32> = WeightMatrix::from_row_vec(3, c);
    let mut marks = MarkMatrix::new(weights.n());
    let mut coverage = Coverage::new(weights.n());

    marks.star((0,0));
    marks.star((1,2));

    let next_step = step3(&weights, &marks, &mut coverage);
    assert_eq!(Step::Step4(Some(2)), next_step);

    assert_eq!(true, coverage.is_col_covered(0));
    assert_eq!(false, coverage.is_col_covered(1));
    assert_eq!(true, coverage.is_col_covered(2));

    assert_eq!(false, coverage.is_row_covered(0));
    assert_eq!(false, coverage.is_row_covered(1));
    assert_eq!(false, coverage.is_row_covered(2));
}

#[test]
fn test_step4_case1() {
    let c = vec![0, 150, 100,
                 50, 250, 0,
                 0, 200, 50];

    let weights: WeightMatrix<i32> = WeightMatrix::from_row_vec(3, c);
    let mut marks = MarkMatrix::new(weights.n());
    let mut coverage = Coverage::new(weights.n());

    marks.star((0,0));
    marks.star((1,2));
    coverage.cover_col(0);
    coverage.cover_col(2);

    let next_step = step4(&weights, &mut marks, &mut coverage);

    assert_eq!(Step::Step6, next_step);

    // coverage did not change.
    assert_eq!(true, coverage.is_col_covered(0));
    assert_eq!(false, coverage.is_col_covered(1));
    assert_eq!(true, coverage.is_col_covered(2));
    assert_eq!(false, coverage.is_row_covered(0));
    assert_eq!(false, coverage.is_row_covered(1));
    assert_eq!(false, coverage.is_row_covered(2));

    // starring did not change.
    assert_eq!(true, marks.is_star((0,0)));
    assert_eq!(false, marks.is_star((0,1)));
    assert_eq!(false, marks.is_star((0,2)));
    assert_eq!(false, marks.is_star((1,0)));
    assert_eq!(false, marks.is_star((1,1)));
    assert_eq!(true, marks.is_star((1,2)));
    assert_eq!(false, marks.is_star((2,0)));
    assert_eq!(false, marks.is_star((2,1)));
    assert_eq!(false, marks.is_star((2,2)));
}

#[test]
fn test_step6() {
    let c = vec![0, 150, 100,
                 50, 250, 0,
                 0, 200, 50];

    let mut weights: WeightMatrix<i32> = WeightMatrix::from_row_vec(3, c);
    let mut marks = MarkMatrix::new(weights.n());
    let mut coverage = Coverage::new(weights.n());

    marks.star((0,0));
    marks.star((1,2));
    coverage.cover_col(0);
    coverage.cover_col(2);

    let next_step = step6(&mut weights, &coverage);

    assert_eq!(Step::Step4(None), next_step);

    let exp = vec![0, 0, 100,
                   50, 100, 0,
                   0, 50, 50];

    assert_eq!(exp, weights.c.into_vec());
}

#[test]
fn test_step4_case2() {
    let c = vec![0, 0, 100,
                 50, 100, 0,
                 0, 50, 50];

    let weights: WeightMatrix<i32> = WeightMatrix::from_row_vec(3, c);
    let mut marks = MarkMatrix::new(weights.n());
    let mut coverage = Coverage::new(weights.n());

    marks.star((0,0));
    marks.star((1,2));
    coverage.cover_col(0);
    coverage.cover_col(2);

    let next_step = step4(&weights, &mut marks, &mut coverage);

    assert_eq!(Step::Step5(2,0), next_step);

    // coverage DID CHANGE!
    assert_eq!(false, coverage.is_col_covered(0));
    assert_eq!(false, coverage.is_col_covered(1));
    assert_eq!(true, coverage.is_col_covered(2));
    assert_eq!(true, coverage.is_row_covered(0));
    assert_eq!(false, coverage.is_row_covered(1));
    assert_eq!(false, coverage.is_row_covered(2));

    // starring DID CHANGE!
    assert_eq!(Mark::Star, marks.get((0,0)));
    assert_eq!(Mark::Prime, marks.get((0,1)));
    assert_eq!(Mark::None, marks.get((0,2)));
    assert_eq!(Mark::None, marks.get((1,0)));
    assert_eq!(Mark::None, marks.get((1,1)));
    assert_eq!(Mark::Star, marks.get((1,2)));
    assert_eq!(Mark::Prime, marks.get((2,0)));
    assert_eq!(Mark::None, marks.get((2,1)));
    assert_eq!(Mark::None, marks.get((2,2)));
}

#[test]
fn test_step5() {
    let c = vec![0, 0, 100,
                 50, 100, 0,
                 0, 50, 50];

    let weights: WeightMatrix<i32> = WeightMatrix::from_row_vec(3, c);
    let mut marks = MarkMatrix::new(weights.n());
    let mut coverage = Coverage::new(weights.n());

    marks.star((0,0));
    marks.prime((0,1));
    marks.star((1,2));
    marks.prime((2,0));

    coverage.cover_col(2);
    coverage.cover_row(0);

    let next_step = step5(&weights, &mut marks, &mut coverage, (2,0));
    assert_eq!(Step::Step3, next_step);

    // coverage DID CHANGE!
    assert_eq!(false, coverage.is_col_covered(0));
    assert_eq!(false, coverage.is_col_covered(1));
    assert_eq!(false, coverage.is_col_covered(2));
    assert_eq!(false, coverage.is_row_covered(0));
    assert_eq!(false, coverage.is_row_covered(1));
    assert_eq!(false, coverage.is_row_covered(2));

    // starring DID CHANGE!
    assert_eq!(Mark::None, marks.get((0,0)));
    assert_eq!(Mark::Star, marks.get((0,1)));
    assert_eq!(Mark::None, marks.get((0,2)));

    assert_eq!(Mark::None, marks.get((1,0)));
    assert_eq!(Mark::None, marks.get((1,1)));
    assert_eq!(Mark::Star, marks.get((1,2)));

    assert_eq!(Mark::Star, marks.get((2,0)));
    assert_eq!(Mark::None, marks.get((2,1)));
    assert_eq!(Mark::None, marks.get((2,2)));
}


#[test]
fn test_1() {
    let c = vec![250, 400, 350,
                 400, 600, 350,
                 200, 400, 250];

    let mut weights: WeightMatrix<i32> = WeightMatrix::from_row_vec(3, c);
    let matching = solve_assignment(&mut weights);

    assert_eq!(vec![(0,1), (1,2), (2,0)], matching);
}
