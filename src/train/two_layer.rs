// < begin copyright > 
// Copyright Ryan Marcus 2020
// 
// See root directory of this project for license terms.
// 
// < end copyright > 
 
use crate::models::*;
use crate::train::{validate, train_model, TrainedRMI};
use crate::train::lower_bound_correction::LowerBoundCorrection;
use log::*;
use superslice::*;

fn error_between(v1: u64, v2: u64) -> u64 {
    return u64::max(v1, v2) - u64::min(v1, v2);
}

fn build_models_from(data: &ModelDataWrapper,
                     top_model: &Box<dyn Model>,
                     model_type: &str,
                     start_idx: usize, end_idx: usize,
                     first_model_idx: usize,
                     num_models: usize) -> Vec<Box<dyn Model>> {

    assert!(end_idx > start_idx);
    assert!(end_idx <= data.len());
    assert!(start_idx <= data.len());

    let empty_data = ModelData::empty();
    let dummy_md = ModelDataWrapper::new(&empty_data);
    let mut leaf_models: Vec<Box<dyn Model>> = Vec::with_capacity(num_models as usize);
    let mut second_layer_data = Vec::with_capacity((end_idx - start_idx) / num_models as usize);
    let mut last_target = first_model_idx;

    let mut bounded_it = data.iter_int_int();
    bounded_it.bound(start_idx, end_idx);
        
    for (x, y) in bounded_it {
        let model_pred = top_model.predict_to_int(x.into()) as usize;
        assert!(top_model.needs_bounds_check() || model_pred < start_idx + num_models,
                "Top model gave an index of {} which is out of bounds of {}",
                model_pred, start_idx + num_models);
        let target = usize::min(first_model_idx + num_models - 1, model_pred);
        assert!(target >= last_target);
        
        if target > last_target {
            // this is the first datapoint for the next leaf model.
            // train the previous leaf model.
            let md = ModelData::IntKeyToIntPos(second_layer_data);
            let container = ModelDataWrapper::new(&md);
            let leaf_model = train_model(model_type, &container);
            leaf_models.push(leaf_model);
            
            
            // leave empty models for any we skipped.
            for _skipped_idx in (last_target+1)..target {
                leaf_models.push(train_model(model_type, &dummy_md));
            }
            assert_eq!(leaf_models.len() + first_model_idx, target);
            
            second_layer_data = Vec::new();
        }
        
        second_layer_data.push((x, y));
        last_target = target;
    }

    // train the last remaining model
    assert!(! second_layer_data.is_empty());
    let md = ModelData::IntKeyToIntPos(second_layer_data);
    let container = ModelDataWrapper::new(&md);
    let leaf_model = train_model(model_type, &container);
    leaf_models.push(leaf_model);
    assert!(leaf_models.len() <= num_models);
    
    // add models at the end with nothing mapped into them
    for _skipped_idx in (last_target+1)..(first_model_idx + num_models) as usize {
        leaf_models.push(train_model(model_type, &dummy_md));
    }
    assert_eq!(num_models as usize, leaf_models.len());
    return leaf_models;
}

pub fn train_two_layer(md_container: &mut ModelDataWrapper,
                       layer1_model: &str, layer2_model: &str,
                       num_leaf_models: u64) -> TrainedRMI {
    validate(&[String::from(layer1_model), String::from(layer2_model)]);

    let num_rows = md_container.len();

    info!("Training top-level {} model layer", layer1_model);
    md_container.set_scale(num_leaf_models as f64 / num_rows as f64);
    let top_model = train_model(layer1_model, &md_container);

    info!("Training second-level {} model layer (num models = {})", layer2_model, num_leaf_models);
    md_container.set_scale(1.0);

    // find a prediction boundary near the middle
    let midpoint_model = num_leaf_models / 2;
    let split_idx = md_container.as_int_int().lower_bound_by(|x| {
        let model_idx = top_model.predict_to_int(x.0.into());
        let model_target = u64::min(num_leaf_models - 1, model_idx);
        return model_target.cmp(&midpoint_model);
    });

    let mut leaf_models = if split_idx >= md_container.len() {
        warn!("All of the data is being mapped into less than half the number of leaf models. Parallelism disabled.");
        build_models_from(&md_container, &top_model, layer2_model,
                          0, md_container.len(), 0,
                          num_leaf_models as usize)
    } else {
    
        let split_idx_target = u64::min(num_leaf_models - 1,
                                        top_model.predict_to_int(md_container.get_key(split_idx).into()))
            as usize;
        

        let first_half_models = split_idx_target as usize;
        let second_half_models = num_leaf_models as usize - split_idx_target as usize;

        let (mut hf1, mut hf2)
            = rayon::join(|| build_models_from(&md_container, &top_model, layer2_model,
                                               0, split_idx - 1,
                                               0,
                                               first_half_models),
                          || build_models_from(&md_container, &top_model, layer2_model,
                                               split_idx, md_container.len(),
                                               split_idx_target,
                                               second_half_models));

        let mut leaf_models = Vec::new();
        leaf_models.append(&mut hf1);
        leaf_models.append(&mut hf2);
        leaf_models
    };

    info!("Computing lower bound stats...");
    let lb_corrections = LowerBoundCorrection::new(
        |x| top_model.predict_to_int(x), num_leaf_models, md_container
    );

    info!("Fixing empty models...");
    // replace any empty model with a model that returns the correct constant
    // (for UB predictions), if the underlying model supports it.
    let mut could_not_replace = false;
    for idx in 0..(num_leaf_models as usize)-1 {
        assert_eq!(lb_corrections.first_key(idx).is_none(),
                   lb_corrections.last_key(idx).is_none());

        if lb_corrections.last_key(idx).is_none() {
            // model is empty!
            let upper_bound = lb_corrections.next_index(idx);
            if !leaf_models[idx].set_to_constant_model(upper_bound) {
                could_not_replace = true;
            }
        }
    }

    if could_not_replace {
        warn!("Some empty models could not be replaced with constants, \
               negative lookup performance may be poor.");
    }
    
    
    info!("Computing last level errors...");
    // evaluate model, compute last level errors
    let mut last_layer_max_l1s = vec![(0, 0) ; num_leaf_models as usize];
    for &(x, y) in md_container.as_int_int() {
        let leaf_idx = top_model.predict_to_int(x.into());
        let target = u64::min(num_leaf_models - 1, leaf_idx) as usize;
        
        let pred = leaf_models[target].predict_to_int(x.into());
        let err = error_between(pred, y);

        let cur_val = last_layer_max_l1s[target];
        last_layer_max_l1s[target] = (cur_val.0 + 1, u64::max(err, cur_val.1));
    }    

    // for lower bound searches, we need to make sure that:
    //   (1) a query for the first key in the next leaf minus one 
    //       includes the key in the next leaf. (upper error)
    //   (2) a query for the last key in the previous leaf plus one
    //       includes the first key after the previous leaf (lower error)
    //       (normally, the first key after the previous leaf is the first
    //        key in this leaf, but not in the case where this leaf has no keys)
    let mut large_corrections = 0;
    for leaf_idx in 0..num_leaf_models as usize {
        let curr_err = last_layer_max_l1s[leaf_idx].1;
        let upper_error = {
            let (idx_of_next, key_of_next) = lb_corrections.next(leaf_idx);
            let pred = leaf_models[leaf_idx].predict_to_int((key_of_next - 1).into());
            error_between(pred, idx_of_next + 1)
        };
        
        let lower_error = {
            let first_key_before = lb_corrections.prev_key(leaf_idx);

            let prev_idx = if leaf_idx == 0 { 0 } else { leaf_idx - 1 };
            let first_idx = lb_corrections.next_index(prev_idx);

            let pred = leaf_models[leaf_idx].predict_to_int((first_key_before + 1).into());
            error_between(pred, first_idx)
        };
          
            
        let new_err = *(&[curr_err, upper_error, lower_error]).iter().max().unwrap()
            + lb_corrections.longest_run(leaf_idx);

        let num_items_in_leaf = last_layer_max_l1s[leaf_idx].0;
        last_layer_max_l1s[leaf_idx] = (num_items_in_leaf, new_err);

        if new_err - curr_err > 512 && num_items_in_leaf > 100 {
            large_corrections += 1;
        }
    }

    if large_corrections > 1 {
        warn!("Of {} models, {} needed large lower bound corrections.",
              num_leaf_models, large_corrections);
    }
                        
    info!("Evaluating two-layer RMI...");
    let (m_idx, m_err) = last_layer_max_l1s
        .iter().enumerate()
        .max_by_key(|(_idx, &x)| x.1).unwrap();
    
    let model_max_error = m_err.1;
    let model_max_error_idx = m_idx;

    let model_avg_error: f64 = last_layer_max_l1s
        .iter().map(|(n, err)| n * err).sum::<u64>() as f64 / num_rows as f64;

    let model_avg_l2_error: f64 = last_layer_max_l1s
        .iter()
        .map(|(n, err)| ((n*err) as f64).powf(2.0) / num_rows as f64).sum::<f64>();
    
    let model_avg_log2_error: f64 = last_layer_max_l1s
        .iter().map(|(n, err)| (*n as f64)*((2*err + 2) as f64).log2()).sum::<f64>() / num_rows as f64;

    let model_max_log2_error: f64 = (model_max_error as f64).log2();
    
    let final_errors = last_layer_max_l1s.into_iter()
        .map(|(_n, err)| err).collect();
    
    return TrainedRMI {
        model_avg_error,
        model_avg_l2_error,
        model_avg_log2_error,
        model_max_error,
        model_max_error_idx,
        model_max_log2_error,
        last_layer_max_l1s: final_errors,
        rmi: vec![vec![top_model], leaf_models],
        models: format!("{},{}", layer1_model, layer2_model),
        branching_factor: num_leaf_models
    };

}
