use log::info;
use rand::Rng;
use std::collections::{HashMap, HashSet};

use crate::gui::BOARD_SIZE;

const DEPTH: usize = 2;
const COLUMN: usize = 15;
const ROW: usize = 15;

type TypeScoreAllArr = Vec<(i32, Vec<(usize, usize)>, (i32, i32))>;

#[derive(Debug, PartialEq, Eq)]
pub enum GameState {
    Idle,
    Human,
    AI,
}

#[derive(Debug)]
struct GobangZobrist {
    black: [[u32; BOARD_SIZE]; BOARD_SIZE],
    white: [[u32; BOARD_SIZE]; BOARD_SIZE],
    hash: u32,
}

impl GobangZobrist {
    pub fn new() -> Self {
        let mut black = [[0; BOARD_SIZE]; BOARD_SIZE];
        let mut white = [[0; BOARD_SIZE]; BOARD_SIZE];
        let mut rng = rand::rng();
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                black[i][j] = rng.random::<u32>();
                white[i][j] = rng.random::<u32>();
            }
        }
        GobangZobrist {
            black,
            white,
            hash: 0,
        }
    }

    pub fn update(&mut self, x: usize, y: usize, is_black: bool) {
        self.hash ^= if is_black {
            self.black[x][y]
        } else {
            self.white[x][y]
        };
    }

    pub fn get_hash(&self) -> u32 {
        self.hash
    }
}

#[derive(Debug)]
pub struct AI {
    ai_steps: Vec<(usize, usize)>,
    ai_steps_st: HashSet<(usize, usize)>,
    human_steps: Vec<(usize, usize)>,
    human_steps_st: HashSet<(usize, usize)>,
    pub all_steps: Vec<(usize, usize)>,
    all_steps_st: HashSet<(usize, usize)>,
    full_steps: HashSet<(usize, usize)>,
    next_step: (usize, usize),

    cut_cnt: usize,
    search_cnt: usize,
    cache_hit: usize,

    pub state: GameState,
    pub depth: usize,

    pub ai_black: bool,
    zobrist: GobangZobrist,
    evaluation_cache_black: HashMap<u32, i32>,
    evaluation_cache_white: HashMap<u32, i32>,
}

const SHAPE_SCORE: &[(i32, &[usize])] = &[
    (50, &[0, 1, 1, 0, 0]),
    (50, &[0, 0, 1, 1, 0]),
    (200, &[1, 1, 0, 1, 0]),
    (500, &[0, 0, 1, 1, 1]),
    (500, &[1, 1, 1, 0, 0]),
    (5000, &[0, 1, 1, 1, 0]),
    (5000, &[0, 1, 0, 1, 1, 0]),
    (5000, &[0, 1, 1, 0, 1, 0]),
    (5000, &[1, 1, 1, 0, 1]),
    (5000, &[1, 1, 0, 1, 1]),
    (5000, &[1, 0, 1, 1, 1]),
    (5000, &[1, 1, 1, 1, 0]),
    (5000, &[0, 1, 1, 1, 1]),
    (50000, &[0, 1, 1, 1, 1, 0]),
    (99999999, &[1, 1, 1, 1, 1]),
];

impl AI {
    pub fn new() -> AI {
        let full_steps: HashSet<(usize, usize)> = (0..ROW)
            .flat_map(|i| (0..COLUMN).map(move |j| (i, j)))
            .collect();
        Self {
            ai_steps: Vec::new(),
            ai_steps_st: HashSet::new(),
            human_steps: Vec::new(),
            human_steps_st: HashSet::new(),
            all_steps: Vec::new(),
            all_steps_st: HashSet::new(),
            full_steps,
            next_step: (0, 0),
            search_cnt: 0,
            cut_cnt: 0,
            cache_hit: 0,
            state: GameState::Idle,
            depth: DEPTH,
            zobrist: GobangZobrist::new(),
            ai_black: false,
            evaluation_cache_black: HashMap::new(),
            evaluation_cache_white: HashMap::new(),
        }
    }

    pub fn ai(&mut self) -> (usize, usize) {
        self.cut_cnt = 0;
        self.search_cnt = 0;
        self.cache_hit = 0;
        self.negamax(true, self.depth, i32::MIN >> 1, i32::MAX >> 1);
        info!(
            "search count: {}; cut count: {}; cache hit: {}",
            self.search_cnt, self.cut_cnt, self.cache_hit
        );
        self.ai_step(self.next_step.0, self.next_step.1);
        self.next_step
    }

    pub fn human_step(&mut self, x: usize, y: usize) {
        self.human_steps.push((x, y));
        self.human_steps_st.insert((x, y));
        self.all_steps.push((x, y));
        self.all_steps_st.insert((x, y));
        self.zobrist.update(x, y, !self.ai_black);
    }

    pub fn ai_step(&mut self, x: usize, y: usize) {
        self.ai_steps.push((x, y));
        self.ai_steps_st.insert((x, y));
        self.all_steps.push((x, y));
        self.all_steps_st.insert((x, y));
        self.zobrist.update(x, y, self.ai_black);
    }

    fn negamax(&mut self, is_ai: bool, depth: usize, mut alpha: i32, beta: i32) -> i32 {
        let mut blank_steps: Vec<(usize, usize)> = self
            .full_steps
            .difference(&self.all_steps_st)
            .copied()
            .collect();
        if AI::game_win(&self.ai_steps_st) || AI::game_win(&self.human_steps_st) || depth == 0 {
            return self.evalution(is_ai, HashSet::from_iter(blank_steps.iter()));
        }
        self.order(&mut blank_steps);
        for (tx, ty) in blank_steps {
            self.search_cnt += 1;

            if !self.has_neighbor(tx, ty) {
                continue;
            }
            if is_ai {
                self.ai_step(tx, ty);
            } else {
                self.human_step(tx, ty);
            }
            let value = -self.negamax(!is_ai, depth - 1, -beta, -alpha);
            if is_ai {
                self.ai_steps.pop();
                self.ai_steps_st.remove(&(tx, ty));
                self.zobrist.update(tx, ty, self.ai_black);
            } else {
                self.human_steps.pop();
                self.human_steps_st.remove(&(tx, ty));
                self.zobrist.update(tx, ty, !self.ai_black);
            }
            self.all_steps.pop();
            self.all_steps_st.remove(&(tx, ty));
            if value > alpha {
                // println!("{};alpha:{};beta:{}", value, alpha, beta);
                if depth == self.depth {
                    self.next_step = (tx, ty);
                }
                if value >= beta {
                    self.cut_cnt += 1;
                    return beta;
                }
                alpha = value;
            }
        }

        alpha
    }

    fn evalution(&mut self, is_ai: bool, blank_steps_st: HashSet<&(usize, usize)>) -> i32 {
        let my_steps = if is_ai {
            &self.ai_steps
        } else {
            &self.human_steps
        };
        let my_steps_st = HashSet::from_iter(my_steps.iter());
        let enemy_steps = if is_ai {
            &self.human_steps
        } else {
            &self.ai_steps
        };
        let enemy_steps_st = HashSet::from_iter(enemy_steps.iter());
        if (is_ai && self.ai_black) || (!is_ai && !self.ai_black) {
            if let Some(val) = self.evaluation_cache_black.get(&self.zobrist.get_hash()) {
                self.cache_hit += 1;
                return *val;
            }
        } else if let Some(val) = self.evaluation_cache_white.get(&self.zobrist.get_hash()) {
            self.cache_hit += 1;
            return *val;
        }
        let mut my_score_all_arr: TypeScoreAllArr = Vec::new();
        let mut my_score = 0;
        for (x, y) in my_steps {
            my_score += self.cal_score(
                (*x as i32, *y as i32),
                (0, 1),
                &my_steps_st,
                &enemy_steps_st,
                &blank_steps_st,
                &mut my_score_all_arr,
            );
            my_score += self.cal_score(
                (*x as i32, *y as i32),
                (1, 0),
                &my_steps_st,
                &enemy_steps_st,
                &blank_steps_st,
                &mut my_score_all_arr,
            );
            my_score += self.cal_score(
                (*x as i32, *y as i32),
                (1, 1),
                &my_steps_st,
                &enemy_steps_st,
                &blank_steps_st,
                &mut my_score_all_arr,
            );
            my_score += self.cal_score(
                (*x as i32, *y as i32),
                (1, -1),
                &my_steps_st,
                &enemy_steps_st,
                &blank_steps_st,
                &mut my_score_all_arr,
            );
        }

        let mut enemy_score = 0;
        let mut enemy_score_all_arr: TypeScoreAllArr = Vec::new();
        for (x, y) in enemy_steps {
            enemy_score += self.cal_score(
                (*x as i32, *y as i32),
                (0, 1),
                &enemy_steps_st,
                &my_steps_st,
                &blank_steps_st,
                &mut enemy_score_all_arr,
            );
            enemy_score += self.cal_score(
                (*x as i32, *y as i32),
                (1, 0),
                &enemy_steps_st,
                &my_steps_st,
                &blank_steps_st,
                &mut enemy_score_all_arr,
            );
            enemy_score += self.cal_score(
                (*x as i32, *y as i32),
                (1, 1),
                &enemy_steps_st,
                &my_steps_st,
                &blank_steps_st,
                &mut enemy_score_all_arr,
            );
            enemy_score += self.cal_score(
                (*x as i32, *y as i32),
                (1, -1),
                &enemy_steps_st,
                &my_steps_st,
                &blank_steps_st,
                &mut enemy_score_all_arr,
            );
        }
        let ret = (my_score as f32 - enemy_score as f32 * 0.1) as i32;
        if (is_ai && self.ai_black) || (!is_ai && !self.ai_black) {
            self.evaluation_cache_black
                .insert(self.zobrist.get_hash(), ret);
        } else {
            self.evaluation_cache_white
                .insert(self.zobrist.get_hash(), ret);
        }
        ret
    }

    fn cal_score(
        &self,
        (x, y): (i32, i32),
        (dx, dy): (i32, i32),
        my_steps_st: &HashSet<&(usize, usize)>,
        enemy_steps_st: &HashSet<&(usize, usize)>,
        blank_steps_st: &HashSet<&(usize, usize)>,
        score_all_arr: &mut TypeScoreAllArr,
    ) -> i32 {
        for (_, shape, delta) in score_all_arr.iter() {
            if dx == delta.0 && dy == delta.1 && shape.contains(&(x as usize, y as usize)) {
                return 0;
            }
        }
        let mut max_score_shape: (i32, Vec<(usize, usize)>, (i32, i32)) = (0, Vec::new(), (0, 0));
        let mut add_score = 0;

        for offset in -5..1 {
            let mut pos: Vec<usize> = Vec::new();
            for d in 0..6 {
                if enemy_steps_st.contains(&(
                    (x + (d + offset) * dx) as usize,
                    (y + (d + offset) * dy) as usize,
                )) {
                    pos.push(2);
                } else if my_steps_st.contains(&(
                    (x + (d + offset) * dx) as usize,
                    (y + (d + offset) * dy) as usize,
                )) {
                    pos.push(1);
                } else if !blank_steps_st.contains(&(
                    (x + (d + offset) * dx) as usize,
                    (y + (d + offset) * dy) as usize,
                )) {
                    // Illegal
                    pos.push(3);
                } else {
                    pos.push(0);
                }
            }
            let tmp_shape5 = vec![pos[0], pos[1], pos[2], pos[3], pos[4]];
            let tmp_shape6 = vec![pos[0], pos[1], pos[2], pos[3], pos[4], pos[5]];
            for (score, shape) in SHAPE_SCORE {
                if (tmp_shape5 == shape.to_vec() || tmp_shape6 == shape.to_vec())
                    && *score > max_score_shape.0
                {
                    max_score_shape = (
                        *score,
                        vec![
                            ((x + offset * dx) as usize, (y + offset * dy) as usize),
                            (
                                (x + (1 + offset) * dx) as usize,
                                (y + (1 + offset) * dy) as usize,
                            ),
                            (
                                (x + (2 + offset) * dx) as usize,
                                (y + (2 + offset) * dy) as usize,
                            ),
                            (
                                (x + (3 + offset) * dx) as usize,
                                (y + (3 + offset) * dy) as usize,
                            ),
                            (
                                (x + (4 + offset) * dx) as usize,
                                (y + (4 + offset) * dy) as usize,
                            ),
                        ],
                        (dx, dy),
                    );
                }
            }
        }

        if !max_score_shape.1.is_empty() {
            for item in score_all_arr.iter() {
                for pt1 in item.1.clone() {
                    for pt2 in max_score_shape.1.clone() {
                        if pt1 == pt2 && max_score_shape.0 > 10 && item.0 > 10 {
                            add_score += item.0 + max_score_shape.0;
                        }
                    }
                }
            }
        }

        score_all_arr.push(max_score_shape.clone());

        add_score + max_score_shape.0
    }

    fn order(&self, blank_steps: &mut Vec<(usize, usize)>) {
        if self.all_steps.is_empty() {
            return;
        }
        let last_pt = self.all_steps.last().unwrap();
        for i in -1..2 {
            for j in -1..2 {
                if i == 0 && j == 0 {
                    continue;
                }
                let pt: (usize, usize) = (
                    (last_pt.0 as i32 + i) as usize,
                    (last_pt.1 as i32 + j) as usize,
                );
                if let Some(idx) = blank_steps.iter().position(|x| *x == pt) {
                    let item = blank_steps.remove(idx);
                    blank_steps.insert(0, item);
                }
            }
        }
    }

    fn has_neighbor(&self, x: usize, y: usize) -> bool {
        for i in -1..2 {
            for j in -1..2 {
                if i == 0 && j == 0 {
                    continue;
                }
                if self
                    .all_steps_st
                    .contains(&((x as i32 + i) as usize, (y as i32 + j) as usize))
                {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_game_over(&mut self) -> bool {
        if AI::game_win(&self.ai_steps_st) {
            self.state = GameState::AI;
            return true;
        } else if AI::game_win(&self.human_steps_st) {
            self.state = GameState::Human;
            return true;
        }
        false
    }

    fn game_win(steps_st: &HashSet<(usize, usize)>) -> bool {
        for i in 0..ROW {
            for j in 0..COLUMN {
                if (j < COLUMN - 4
                    && steps_st.contains(&(i, j))
                    && steps_st.contains(&(i, j + 1))
                    && steps_st.contains(&(i, j + 2))
                    && steps_st.contains(&(i, j + 3))
                    && steps_st.contains(&(i, j + 4)))
                    || (i < ROW - 4
                        && steps_st.contains(&(i, j))
                        && steps_st.contains(&(i + 1, j))
                        && steps_st.contains(&(i + 2, j))
                        && steps_st.contains(&(i + 3, j))
                        && steps_st.contains(&(i + 4, j)))
                    || (i < ROW - 4
                        && j < COLUMN - 4
                        && steps_st.contains(&(i, j))
                        && steps_st.contains(&(i + 1, j + 1))
                        && steps_st.contains(&(i + 2, j + 2))
                        && steps_st.contains(&(i + 3, j + 3))
                        && steps_st.contains(&(i + 4, j + 4)))
                    || (i < ROW - 4
                        && j > 3
                        && steps_st.contains(&(i, j))
                        && steps_st.contains(&(i + 1, j - 1))
                        && steps_st.contains(&(i + 2, j - 2))
                        && steps_st.contains(&(i + 3, j - 3))
                        && steps_st.contains(&(i + 4, j - 4)))
                {
                    return true;
                }
            }
        }
        false
    }
}
