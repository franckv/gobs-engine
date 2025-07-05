use std::collections::VecDeque;

pub fn sort_bfs<T>(nodes: &Vec<T>, edges: &Vec<(usize, usize)>) -> Vec<usize> {
    let mut sorted = Vec::new();

    let mut adjacent = (0..nodes.len())
        .map(|_| Vec::new())
        .collect::<Vec<Vec<usize>>>();
    let mut incoming: Vec<usize> = (0..nodes.len()).map(|_| 0).collect::<Vec<usize>>();

    for &(src, dst) in edges {
        adjacent[src].push(dst);
        incoming[dst] += 1;
    }

    let mut start = VecDeque::new();
    for i in 0..nodes.len() {
        if incoming[i] == 0 {
            start.push_back(i);
        }
    }

    while !start.is_empty() {
        let node = start.pop_front().unwrap();

        sorted.push(node);

        for &i in &adjacent[node] {
            incoming[i] -= 1;

            if incoming[i] == 0 {
                start.push_back(i);
            }
        }
    }

    sorted
}

pub fn sort_dfs<T>(nodes: &Vec<T>, edges: &Vec<(usize, usize)>) -> Vec<usize> {
    let mut visited = (0..nodes.len()).map(|_| false).collect::<Vec<bool>>();

    let mut adjacent = (0..nodes.len())
        .map(|_| Vec::new())
        .collect::<Vec<Vec<usize>>>();

    for &(src, dst) in edges {
        adjacent[src].push(dst);
    }

    let mut sorted = Vec::new();

    for i in 0..nodes.len() {
        if !visited[i] {
            dfs(i, &mut visited, &adjacent, &mut sorted);
        }
    }

    sorted.reverse();

    sorted
}

fn dfs(index: usize, visited: &mut Vec<bool>, adjacent: &Vec<Vec<usize>>, sorted: &mut Vec<usize>) {
    visited[index] = true;

    for &u in &adjacent[index] {
        if !visited[u] {
            dfs(u, visited, adjacent, sorted);
        }
    }
    sorted.push(index);
}
