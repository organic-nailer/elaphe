use crate::{
    grammar::{EPSILON, END},
    parser_generator_lr0::{LRProductionRuleData, ParserGeneratorLR0, ProductionRuleData},
    hashable_set::HashableSet,
};
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

pub type State = String;
pub type Token = &'static str;
type Kernel = (String, LRProductionRuleData);

pub fn generate_parser(
    rules: &Vec<ProductionRuleData>,
    start_symbol: &'static str
) -> TransitionMap {
    let parser_generator_lr0 = ParserGeneratorLR0::new(rules, start_symbol);

    let first_map = calc_first(rules, &parser_generator_lr0);

    let closure_map = calc_goto_map(start_symbol, rules, &first_map, &parser_generator_lr0);

    let transition_map = calc_transition_map(&parser_generator_lr0, &closure_map);

    TransitionMap {
        transitions: transition_map,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionMap {
    pub transitions: HashMap<(String, String), TransitionData>,
}

/// 
/// right_token_list + suffixesのToken列のfirst集合を計算する
/// 
/// # Arguments
/// - right_token_list: Token列
/// - suffixes: right_token_listの後ろに存在する可能性のあるTokenの集合
/// - first_map: 現時点で計算済みのfirst集合。Tokenごとのfirst集合を格納
/// - parser_generator_lr0: ParserGeneratorLR0
fn get_first(
    right_token_list: &Vec<Token>,
    suffixes: HashSet<Token>,
    first_map: &HashMap<Token, HashSet<Token>>,
    parser_generator_lr0: &ParserGeneratorLR0,
) -> HashSet<Token> {
    // right_token_listが空もしくは空文字の場合はsuffixesを返す
    if right_token_list.is_empty() {
        return suffixes;
    }
    let value_first = *right_token_list.first().unwrap();
    if value_first == EPSILON && right_token_list.len() == 1 {
        return suffixes;
    }
    // right_token_listの先頭が終端記号の場合はそのTokenを返す
    if parser_generator_lr0.terminal_tokens.contains(value_first) || value_first == END {
        return HashSet::from([value_first]);
    }

    // right_token_listの先頭が非終端記号の場合はそのTokenのfirst集合を返す
    if let Some(current_first_set) = first_map.get(value_first) {
        let current_first_set: HashSet<&str> = current_first_set.iter().map(|v| *v).collect();
        // 空集合が含まれていない場合はそのTokenのfirst集合を返す
        if !current_first_set.contains(&EPSILON) {
            return current_first_set;
        }

        // current_first_setに空集合が含まれるかつright_token_listがTokenを1つしか持たない場合
        if right_token_list.len() == 1 {
            if !suffixes.is_empty() {
                // 空集合を除いてsuffixesとの和集合を返す
                let mut right_token_list = current_first_set.clone();
                right_token_list.remove(EPSILON);
                return suffixes.union(&right_token_list).map(|v| *v).collect();
            }

            return current_first_set.iter().map(|v| *v).collect();
        }

        // Tokenが2つ以上ある場合は、空集合を除いたcurrent_first_setと
        // right_token_list[1..]のfirst集合の和集合を返す
        let mut current_first_set = current_first_set.clone();
        current_first_set.remove(EPSILON);
        let first_alpha = get_first(
            &right_token_list[1..].to_vec(),
            suffixes,
            first_map,
            parser_generator_lr0,
        );
        return current_first_set.union(&first_alpha).map(|v| *v).collect();
    }

    return HashSet::new();
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

fn calc_first(
    rules: &Vec<ProductionRuleData>,
    parser_generator_lr0: &ParserGeneratorLR0,
) -> HashMap<Token, HashSet<Token>> {
    let mut first_map: HashMap<Token, HashSet<Token>> = HashMap::new();

    let mut updated = true;
    while updated {
        updated = false;
        for rule in rules {

            // first_alphaはrule.rightの最初の文字列のfirst集合
            // もし既存のrule.leftのfirst集合に含まれないTokenがあれば、
            // first_mapに追加する
            // 追加されたら、updatedをtrueにする
            let first_alpha = get_first(&rule.right, HashSet::new(), &first_map, parser_generator_lr0);
            match first_map.get(rule.left) {
                Some(first_x) => {
                    let diff: HashSet<&str> = first_alpha.difference(first_x).map(|v| *v).collect();
                    if !diff.is_empty() {
                        updated = true;
                        add_first(rule.left, diff, &mut first_map);
                    }
                },
                None => {
                    updated = true;
                    add_first(rule.left, first_alpha, &mut first_map);
                }
            };
        }
    }

    first_map
}

fn calc_goto_map(
    start_symbol: Token,
    rules: &Vec<ProductionRuleData>,
    first_map: &HashMap<Token, HashSet<Token>>,
    parser_generator_lr0: &ParserGeneratorLR0,
) -> HashMap<HashableSet<LALR1ProductionRuleData>, State> {
    let mut closure_map: HashMap<HashableSet<LALR1ProductionRuleData>, State> = HashMap::new();
    let initial_grammar = ProductionRuleData {
        left: "[START]",
        right: vec![start_symbol],
    };
    let mut extended_rules = rules.clone();
    extended_rules.push(initial_grammar.clone());

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
            &extended_rules, 
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
    match follows_map.get_mut(&("I0".to_string(), initial_grammar.to_lr())) {
        Some(follow) => {
            follow.insert(END);
        }
        None => {}
    }
    updated_kernels.insert(("I0".to_string(), initial_grammar.to_lr()));

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

    for (state, rules) in kernel_map {
        closure_map.insert(
            get_closure(
                rules.iter().map(|r| r.to_lalr1(follows_map.get(&(state.to_string(), r.clone())).unwrap().iter().map(|r| *r).collect())).collect(),
                &extended_rules, 
                first_map, 
                parser_generator_lr0),
            state,
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
            TransitionData {
                kind: TransitionKind::Shift,
                target: Some(target_state.to_string()),
                rule: None,
            },
        );
    }

    for (kernel, state) in closure_map {
        for rule in kernel.iter().filter(|r| r.reducible) {
            for token in rule.follow.iter() {
                if transition_map.contains_key(&(state.to_string(), token.to_string())) {
                    // Shift-Reduce conflict
                    // priortize shift by default
                    continue;
                }

                transition_map.insert(
                    (state.to_string(), token.to_string()),
                    TransitionData {
                        kind: TransitionKind::Reduce,
                        target: None,
                        rule: Some(SerializableRule::from_rule(rule.to_rule())),
                    },
                );
            }
        }
        if kernel.iter().any(|r| {
            r.right == vec![parser_generator_lr0.start_symbol]
                && r.follow.contains(&END)
                && r.dot == r.right.len()
        }) {
            transition_map.insert(
                (state.to_string(), END.to_string()),
                TransitionData {
                    kind: TransitionKind::Accept,
                    target: None,
                    rule: None,
                },
            );
        }
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
        for (rule, follow) in result.clone().iter() {
            if rule.reducible {
                continue;
            }
            let target = rule.right[rule.dot];
            if parser_generator_lr0.non_terminal_tokens.contains(target) {
                let first_set = get_first(
                    &rule.right[rule.dot + 1..].to_vec(),
                    follow.set.clone().into_iter().collect(),
                    first_map,
                    parser_generator_lr0,
                );
                for g_rule in grammar_rules.iter().filter(|r| r.left == target) {
                    let r = g_rule.to_lalr1(HashableSet::new());
                    match result.get_mut(&r) {
                        Some(follow) => {
                            for f in first_set.clone() {
                                if follow.insert(f) {
                                    updated = true;
                                }
                            }
                        }
                        None => {
                            if first_set.len() > 0 {
                                updated = true;
                            }
                            result.insert(r, HashableSet::from_iter(first_set.clone()));
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
pub enum TransitionKind {
    Shift,
    Reduce,
    Accept,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransitionData {
    pub kind: TransitionKind,
    pub target: Option<String>,
    pub rule: Option<SerializableRule>,
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
