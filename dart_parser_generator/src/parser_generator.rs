use crate::{
    grammar::{EPSILON, END},
    parser_generator_lr0::{LRProductionRuleData, ParserGeneratorLR0, ProductionRuleData},
    hashable_set::HashableSet,
    export_transitions::{export_transitions, export_closures},
};
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

pub type State = String;
pub type Token = &'static str;
type Kernel = (String, LRProductionRuleData);

pub fn generate_parser(
    rules: &Vec<ProductionRuleData>,
    start_symbol: &'static str,
    output_transitions: bool,
) -> TransitionMap {
    let initial_grammar = ProductionRuleData {
        left: "[START]",
        right: vec![start_symbol, END],
    };
    let mut extended_rules = rules.clone();
    extended_rules.push(initial_grammar.clone());

    let parser_generator_lr0 = ParserGeneratorLR0::new(&extended_rules, &initial_grammar, start_symbol);

    let first_map = calc_first_pure(&extended_rules, &parser_generator_lr0);

    let closure_map = calc_goto_map(&extended_rules, &first_map, &parser_generator_lr0);

    if output_transitions {
        export_closures(&closure_map, "closure.yaml").unwrap();
    }

    let transition_map = calc_transition_map(&parser_generator_lr0, &closure_map);

    let result = TransitionMap {
        transitions: transition_map,
    };

    if output_transitions {
        export_transitions(&result, &closure_map, &parser_generator_lr0, "transitions.csv").unwrap();
    }

    result
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionMap {
    pub transitions: HashMap<(String, String), TransitionData>,
}

/// 
/// right_token_listのToken列のfirst集合を計算する
/// 
/// # Arguments
/// - token_list: Token列
/// - first_map: 現時点で計算済みのfirst集合。Tokenごとのfirst集合を格納
/// - parser_generator_lr0: ParserGeneratorLR0
fn get_first_pure(
    token_list: &Vec<Token>,
    first_map: &HashMap<Token, HashSet<Token>>,
    parser_generator_lr0: &ParserGeneratorLR0
) -> HashSet<Token> {
    assert!(token_list.len() > 0);
    // First(ε) = {ε}
    if token_list.len() == 1 && token_list[0] == EPSILON {
        return HashSet::from([EPSILON]);
    }

    // First(aα) = {a} if a is terminal token
    if parser_generator_lr0.terminal_tokens.contains(&token_list[0]) {
        return HashSet::from([token_list[0]]);
    }

    // First(Yα) = First(Y) if First(Y) does not contain ε
    // First(Yα) = First(Y) - {ε} + First(α) if First(Y) contains ε
    let mut y_first_set = match first_map.get(&token_list[0]) {
        Some(v) => v.clone(),
        None => HashSet::new(), // if First(Y) has not been calculated yet, return empty set
    };
    if y_first_set.contains(EPSILON) {
        if token_list.len() == 1 {
            y_first_set
        }
        else {
            y_first_set.remove(EPSILON);
            let alpha_first_set = get_first_pure(&token_list[1..].to_vec(), first_map, parser_generator_lr0);
            y_first_set.union(&alpha_first_set).map(|v| *v).collect()
        }
    }
    else {
        y_first_set
    }
}

fn add_first(
    key: Token,
    set: HashSet<Token>,
    first_map: &mut HashMap<Token, HashSet<Token>>,
) {
    if !first_map.contains_key(key) {
        first_map.insert(key, HashSet::new());
    }
    let current_first = first_map.get(key).unwrap();
    let union: HashSet<&str> = current_first.union(&set).map(|v| *v).collect();
    first_map.insert(key, union);
}

fn calc_first_pure(
    rules: &Vec<ProductionRuleData>,
    parser_generator_lr0: &ParserGeneratorLR0
) -> HashMap<Token, HashSet<Token>> {
    let mut first_map: HashMap<Token, HashSet<Token>> = HashMap::new();

    let mut updated = true;
    while updated {
        updated = false;
        for rule in rules {
            // if X -> α, add First(α) to First(X)
            let first_alpha = get_first_pure(&rule.right, &first_map, parser_generator_lr0);
            match first_map.get(rule.left) {
                Some(current_first_set) => {
                    let current_first_set: HashSet<&str> = current_first_set.iter().map(|v| *v).collect();
                    if first_alpha.difference(&current_first_set).count() > 0 {
                        updated = true;
                        add_first(rule.left, first_alpha, &mut first_map);
                    }
                },
                None => {
                    updated = true;
                    add_first(rule.left, first_alpha, &mut first_map);
                }
            }
        }
    }

    first_map
}

fn calc_goto_map(
    rules: &Vec<ProductionRuleData>,
    first_map: &HashMap<Token, HashSet<Token>>,
    parser_generator_lr0: &ParserGeneratorLR0,
) -> HashMap<HashableSet<LALR1ProductionRuleData>, State> {
    let mut closure_map: HashMap<HashableSet<LALR1ProductionRuleData>, State> = HashMap::new();

    let kernel_map = parser_generator_lr0.get_kernel_map();
    let mut kernels: HashSet<Kernel> = HashSet::new();
    let mut follows_map: HashMap<Kernel, HashSet<Token>> = HashMap::new();
    for (state, kernel) in &kernel_map {
        for rule in kernel.iter() {
            kernels.insert((state.clone(), rule.clone()));
            follows_map.insert((state.clone(), rule.clone()), HashSet::new());
        }
    }

    // KernelごとのClosureを求める
    // 先読み記号の伝搬先をpropagate_mapに記録する
    let mut propagate_map: HashMap<Kernel, HashSet<Kernel>> = HashMap::new();
    let mut updated_kernels: HashSet<Kernel> = HashSet::new();

    for (kernel_state, kernel_rule) in kernels {
        let mut follow: HashableSet<Token> = HashableSet::new();
        follow.insert("####");
        let closure = get_closure(
            HashSet::from([kernel_rule.to_lalr1(follow)]), 
            rules, 
            first_map, 
            parser_generator_lr0);
        let mut prop_set: HashSet<Kernel> = HashSet::new();

        for target_rule in closure.iter().filter(|r| !r.reducible) {
            let shift_target_token = target_rule.right[target_rule.dot];
            let shifted_rule = target_rule.shift().unwrap().to_lr();
            let next_state = parser_generator_lr0
                .goto_table
                .get(&(kernel_state.to_string(), shift_target_token))
                .unwrap();

            for follow_token in target_rule.follow.iter() {
                if *follow_token == "####" {
                    // 先読み記号に####が含まれる場合、伝搬先を登録する
                    prop_set.insert((next_state.clone(), shifted_rule.clone()));
                } else {
                    // ####以外の先読み記号がある場合は、follows_mapに追加する
                    follows_map
                        .get_mut(&(next_state.clone(), shifted_rule.clone()))
                        .unwrap()
                        .insert(&follow_token);
                    // follow_mapが更新されたら、updated_kernelsに追加する
                    updated_kernels.insert((next_state.clone(), shifted_rule.clone()));
                }
            }
        }
        propagate_map.insert((kernel_state, kernel_rule), prop_set);
    }

    while !updated_kernels.is_empty() {
        let updated = updated_kernels.clone();
        updated_kernels.clear();
        for kernel in updated {
            let kernel_follows = follows_map.get(&kernel);
            if kernel_follows.is_none() {
                continue;
            }
            let kernel_follows = kernel_follows.unwrap().clone();
            if let Some(prop_set) = propagate_map.get(&kernel) {
                for prop in prop_set {
                    match follows_map.get_mut(prop) {
                        Some(follow) => {
                            let old_len = follow.len();
                            for f in &kernel_follows {
                                follow.insert(f);
                            }
                            if old_len != follow.len() {
                                updated_kernels.insert(prop.clone());
                            }
                        }
                        None => {}
                    }
                }
            }
        }
    }

    for (kernel_state, kernek_rules) in kernel_map {
        closure_map.insert(
            get_closure(
                kernek_rules
                    .iter()
                    .map(|r| r.to_lalr1(follows_map.get(&(kernel_state.to_string(), r.clone())).unwrap().iter().map(|r| *r).collect()))
                    .collect(),
                rules, 
                first_map, 
                parser_generator_lr0),
            kernel_state,
        );
    }

    closure_map
}

fn calc_transition_map(
    parser_generator_lr0: &ParserGeneratorLR0,
    closure_map: &HashMap<HashableSet<LALR1ProductionRuleData>, String>,
) -> HashMap<(String, String), TransitionData> {
    let mut transition_map: HashMap<(String, String), TransitionData> = HashMap::new();

    for (key, target_state) in &parser_generator_lr0.goto_table {
        if transition_map.contains_key(&(key.0.clone(), key.1.to_string())) {
            panic!(
                "LALR1 collision. Transition map already contains key: {:?}",
                key
            );
        }

        transition_map.insert(
            (key.0.clone(), key.1.to_string()),
            TransitionData::Shift {
                target: target_state.to_string(),
            },
        );
    }

    let mut error_transitions: Vec<ErrorTransition> = Vec::new();

    for (kernel, state) in closure_map {
        for rule in kernel.iter().filter(|r| r.reducible) {
            for token in rule.follow.iter() {
                if transition_map.contains_key(&(state.to_string(), token.to_string())) {
                    let transition = transition_map
                        .get(&(state.to_string(), token.to_string()))
                        .unwrap();
                    match transition {
                        TransitionData::Shift { target } => {
                            // Shift-Reduce conflict
                            // priortize shift by default
                            if *token == "else" { continue }
                            if *token == "import" { continue }
                            if *token == "on" { continue }
                            if *token == "(" && rule.left == "Selector" { continue }

                            error_transitions.push(ErrorTransition::ShiftReduce {
                                state: state.to_string(),
                                follow_token: token.to_string(),
                                conflict_reduce_rule: SerializableRule::from_rule(rule.to_rule()),
                                conflict_shift_state: target.clone(),
                            });
        
                            continue
                        }
                        TransitionData::Reduce { rule: conflicted_rule } => {
                            // Reduce-Reduce conflict
                            if *token == "as" {
                                transition_map.insert(
                                    (state.to_string(), token.to_string()),
                                    TransitionData::ReduceReduceConflict {
                                        rules: vec![
                                            SerializableRule::from_rule(rule.to_rule()),
                                            conflicted_rule.clone(),
                                        ],
                                    },
                                );
                                continue
                            }

                            error_transitions.push(ErrorTransition::ReduceReduce {
                                state: state.to_string(),
                                follow_token: token.to_string(),
                                conflict_reduce_rule_1: SerializableRule::from_rule(rule.to_rule()),
                                conflict_reduce_rule_2: conflicted_rule.clone(),
                            });
                        }
                        TransitionData::Accept => {}
                        TransitionData::ReduceReduceConflict { rules } => {
                            let mut cloned_rules = rules.clone();
                            cloned_rules.push(SerializableRule::from_rule(rule.to_rule()));
                            transition_map.insert(
                                (state.to_string(), token.to_string()),
                                TransitionData::ReduceReduceConflict {
                                    rules: cloned_rules,
                                },
                            );
                        }
                    }
                }

                transition_map.insert(
                    (state.to_string(), token.to_string()),
                    TransitionData::Reduce {
                        rule: SerializableRule::from_rule(rule.to_rule()),
                    },
                );
            }
        }
        // if the kernel contains a rule that can shift END, then the state is an accept state
        if kernel.iter().any(|r| {
            !r.reducible && r.right[r.dot] == END
        }) {
            transition_map.insert(
                (state.to_string(), END.to_string()),
                TransitionData::Accept
            );
        }
    }

    if !error_transitions.is_empty() {
        panic!("Unhandled conflicts: {:?}", error_transitions);
    }

    transition_map
}

fn get_closure(
    input: HashSet<LALR1ProductionRuleData>,
    grammar_rules: &Vec<ProductionRuleData>,
    first_map: &HashMap<Token, HashSet<Token>>,
    parser_generator_lr0: &ParserGeneratorLR0,
) -> HashableSet<LALR1ProductionRuleData> {
    let mut result: HashMap<LALR1ProductionRuleData, HashableSet<Token>> = HashMap::new();

    for rule in input {
        let follow = rule.follow.clone();
        let mut rule = rule.clone();
        rule.follow = HashableSet::new();
        result.insert(rule, follow);
    }

    let mut updated = true;
    while updated {
        updated = false;
        for (rule, base_follow) in result.clone().iter() {
            if rule.reducible {
                continue;
            }
            let target = rule.right[rule.dot];
            if parser_generator_lr0.non_terminal_tokens.contains(target) {
                // A -> α target β
                // add First(β) - {ε} to Follow(target)
                // if ε in First(β), add Follow(A) to Follow(target)
                let mut first_set = if rule.right.len() > rule.dot + 1 { 
                    get_first_pure(
                        &rule.right[rule.dot + 1..].to_vec(), 
                        first_map, 
                        parser_generator_lr0)
                } else {
                    HashSet::from([EPSILON])
                };

                let first_has_epsilon = first_set.contains(EPSILON);
                first_set.remove(EPSILON);
                // if target == "ExpressionStatement" {
                //     first_set.remove("{");
                // }
                // let first_set = get_first(
                //     &rule.right[rule.dot + 1..].to_vec(),
                //     follow.set.clone().into_iter().collect(),
                //     first_map,
                //     parser_generator_lr0,
                // );
                for g_rule in grammar_rules.iter().filter(|r| r.left == target) {
                    let r = g_rule.to_lalr1(HashableSet::new());
                    match result.get_mut(&r) {
                        Some(follow) => {
                            for token in first_set.clone() {
                                if follow.insert(token) {
                                    updated = true;
                                }
                            }
                            if first_has_epsilon {
                                for token in base_follow.set.clone() {
                                    if follow.insert(token) {
                                        updated = true;
                                    }
                                }
                            }
                        }
                        None => {
                            let mut new_follow_set = first_set.clone();
                            if first_has_epsilon {
                                new_follow_set.extend(base_follow.set.clone());
                            }
                            if new_follow_set.len() > 0 {
                                updated = true;
                            }
                            result.insert(r, HashableSet::from_iter(new_follow_set));
                        }
                    }
                }
            }
        }
    }

    result
        .iter()
        .map(|(rule, follow)| {
            let mut rule = rule.clone();
            rule.follow = follow.clone();
            rule
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct LALR1ProductionRuleData {
    pub left: &'static str,
    pub right: Vec<&'static str>,
    pub dot: usize,
    pub reducible: bool,
    pub follow: HashableSet<Token>,
}

impl LALR1ProductionRuleData {
    pub fn shift(&self) -> Result<LALR1ProductionRuleData, String> {
        if self.reducible {
            return Err("Cannot shift a reducible rule.".to_string());
        }
        Ok(LALR1ProductionRuleData {
            left: self.left,
            right: self.right.clone(),
            dot: self.dot + 1,
            reducible: self.dot + 1 >= self.right.len(),
            follow: self.follow.clone(),
        })
    }

    pub fn to_rule(&self) -> ProductionRuleData {
        ProductionRuleData {
            left: self.left,
            right: self.right.clone(),
        }
    }

    pub fn to_lr(&self) -> LRProductionRuleData {
        LRProductionRuleData {
            left: self.left,
            right: self.right.clone(),
            dot: self.dot,
            reducible: self.reducible,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TransitionData {
    Shift {
        target: String,
    },
    Reduce {
        rule: SerializableRule,
    },
    Accept,
    ReduceReduceConflict {
        rules: Vec<SerializableRule>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableRule {
    pub left: String,
    pub right: Vec<String>,
}

impl SerializableRule {
    pub fn from_rule(rule: ProductionRuleData) -> SerializableRule {
        SerializableRule {
            left: rule.left.to_string(),
            right: rule.right.iter().map(|s| s.to_string()).collect(),
        }
    }
}

#[derive(Clone, Debug)]
enum ErrorTransition {
    ShiftReduce {
        state: String,
        follow_token: String,
        conflict_reduce_rule: SerializableRule,
        conflict_shift_state: String,
    },
    ReduceReduce {
        state: String,
        follow_token: String,
        conflict_reduce_rule_1: SerializableRule,
        conflict_reduce_rule_2: SerializableRule,
    },
}
