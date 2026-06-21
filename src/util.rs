use std::cmp;

pub fn get_mut_two<T>(vec: &mut [T], i: usize, j: usize) -> Option<(&mut T, &mut T)> {
    if i == j {return None;}
    let (first, second) = vec.split_at_mut(cmp::max(i, j));
    if i < j {
        Some((&mut first[i], &mut second[0]))
    } else {
        Some((&mut second[0], &mut first[j]))
    }
}