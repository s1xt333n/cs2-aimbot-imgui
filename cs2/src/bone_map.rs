use std::collections::HashMap;

/// BodyPart represents different anatomical regions of a player model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BodyPart {
    Head,
    Neck,
    UpperSpine,
    MiddleSpine,
    LowerSpine,
    Pelvis,
    LeftShoulder,
    LeftElbow,
    LeftHand,
    RightShoulder,
    RightElbow,
    RightHand,
    LeftHip,
    LeftKnee,
    LeftFoot,
    RightHip,
    RightKnee,
    RightFoot,
}

/// Structure to hold bone mappings for different model types
#[derive(Debug, Clone)]
pub struct BoneMap {
    // Maps body part to a list of potential bone names in order of preference
    bone_mappings: HashMap<BodyPart, Vec<String>>,
}

impl Default for BoneMap {
    fn default() -> Self {
        let mut bone_mappings = HashMap::new();
        
        // Head bones - different models might use different naming conventions
        bone_mappings.insert(
            BodyPart::Head,
            vec![
                "head".to_string(),
                "head_0".to_string(),
                "face".to_string(),
                "bip_head".to_string(),
                "ValveBiped.Bip01_Head1".to_string(),
            ],
        );
        
        // Neck bones
        bone_mappings.insert(
            BodyPart::Neck,
            vec![
                "neck_0".to_string(), 
                "neck_01".to_string(),
                "neck".to_string(),
                "bip_neck".to_string(),
                "ValveBiped.Bip01_Neck1".to_string(),
            ],
        );
        
        // Upper spine (chest)
        bone_mappings.insert(
            BodyPart::UpperSpine,
            vec![
                "spine_2".to_string(),
                "spine_02".to_string(),
                "spine2".to_string(),
                "chest".to_string(),
                "bip_spine_2".to_string(),
                "ValveBiped.Bip01_Spine2".to_string(),
            ],
        );
        
        // Middle spine
        bone_mappings.insert(
            BodyPart::MiddleSpine,
            vec![
                "spine_1".to_string(),
                "spine_01".to_string(),
                "spine1".to_string(),
                "bip_spine_1".to_string(),
                "ValveBiped.Bip01_Spine1".to_string(),
            ],
        );
        
        // Lower spine (near pelvis)
        bone_mappings.insert(
            BodyPart::LowerSpine,
            vec![
                "spine_0".to_string(),
                "spine_00".to_string(),
                "spine0".to_string(),
                "spine".to_string(),
                "bip_spine_0".to_string(),
                "ValveBiped.Bip01_Spine".to_string(),
            ],
        );
        
        // Pelvis
        bone_mappings.insert(
            BodyPart::Pelvis,
            vec![
                "pelvis".to_string(),
                "hips".to_string(),
                "bip_pelvis".to_string(),
                "ValveBiped.Bip01_Pelvis".to_string(),
            ],
        );
        
        // Left arm bones
        bone_mappings.insert(
            BodyPart::LeftShoulder,
            vec![
                "shoulder_l".to_string(),
                "clavicle_l".to_string(),
                "l_shoulder".to_string(),
                "bip_collar_l".to_string(),
                "ValveBiped.Bip01_L_Clavicle".to_string(),
            ],
        );
        
        bone_mappings.insert(
            BodyPart::LeftElbow,
            vec![
                "elbow_l".to_string(),
                "arm_l".to_string(),
                "l_elbow".to_string(),
                "bip_upperArm_l".to_string(),
                "ValveBiped.Bip01_L_UpperArm".to_string(),
            ],
        );
        
        bone_mappings.insert(
            BodyPart::LeftHand,
            vec![
                "hand_l".to_string(),
                "l_hand".to_string(),
                "bip_hand_l".to_string(),
                "ValveBiped.Bip01_L_Hand".to_string(),
            ],
        );
        
        // Right arm bones
        bone_mappings.insert(
            BodyPart::RightShoulder,
            vec![
                "shoulder_r".to_string(),
                "clavicle_r".to_string(),
                "r_shoulder".to_string(),
                "bip_collar_r".to_string(),
                "ValveBiped.Bip01_R_Clavicle".to_string(),
            ],
        );
        
        bone_mappings.insert(
            BodyPart::RightElbow,
            vec![
                "elbow_r".to_string(),
                "arm_r".to_string(),
                "r_elbow".to_string(),
                "bip_upperArm_r".to_string(),
                "ValveBiped.Bip01_R_UpperArm".to_string(),
            ],
        );
        
        bone_mappings.insert(
            BodyPart::RightHand,
            vec![
                "hand_r".to_string(),
                "r_hand".to_string(),
                "bip_hand_r".to_string(),
                "ValveBiped.Bip01_R_Hand".to_string(),
            ],
        );
        
        // Left leg bones
        bone_mappings.insert(
            BodyPart::LeftHip,
            vec![
                "thigh_l".to_string(),
                "hip_l".to_string(),
                "l_hip".to_string(),
                "bip_hip_l".to_string(),
                "ValveBiped.Bip01_L_Thigh".to_string(),
            ],
        );
        
        bone_mappings.insert(
            BodyPart::LeftKnee,
            vec![
                "knee_l".to_string(),
                "calf_l".to_string(),
                "l_knee".to_string(),
                "bip_knee_l".to_string(),
                "ValveBiped.Bip01_L_Calf".to_string(),
            ],
        );
        
        bone_mappings.insert(
            BodyPart::LeftFoot,
            vec![
                "foot_l".to_string(),
                "l_foot".to_string(),
                "bip_foot_l".to_string(),
                "ValveBiped.Bip01_L_Foot".to_string(),
            ],
        );
        
        // Right leg bones
        bone_mappings.insert(
            BodyPart::RightHip,
            vec![
                "thigh_r".to_string(),
                "hip_r".to_string(),
                "r_hip".to_string(),
                "bip_hip_r".to_string(),
                "ValveBiped.Bip01_R_Thigh".to_string(),
            ],
        );
        
        bone_mappings.insert(
            BodyPart::RightKnee,
            vec![
                "knee_r".to_string(),
                "calf_r".to_string(),
                "r_knee".to_string(),
                "bip_knee_r".to_string(),
                "ValveBiped.Bip01_R_Calf".to_string(),
            ],
        );
        
        bone_mappings.insert(
            BodyPart::RightFoot,
            vec![
                "foot_r".to_string(),
                "r_foot".to_string(),
                "bip_foot_r".to_string(),
                "ValveBiped.Bip01_R_Foot".to_string(),
            ],
        );
        
        Self { bone_mappings }
    }
}

impl BoneMap {
    /// Creates a new BoneMap with default mappings
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Finds the bone index for a given body part in a model's bone array
    pub fn find_bone_index(&self, body_part: BodyPart, bone_names: &[String]) -> Option<usize> {
        if let Some(possible_names) = self.bone_mappings.get(&body_part) {
            for name in possible_names {
                if let Some(index) = bone_names.iter().position(|bone_name| bone_name == name) {
                    return Some(index);
                }
            }
        }
        None
    }
    
    /// Gets a vector of bone names for a specific body part
    pub fn get_bone_names(&self, body_part: BodyPart) -> Vec<String> {
        self.bone_mappings.get(&body_part)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Gets the primary (most preferred) bone name for a body part
    pub fn get_primary_bone_name(&self, body_part: BodyPart) -> Option<String> {
        self.bone_mappings.get(&body_part)
            .and_then(|names| names.first().cloned())
    }
    
    /// Gets a map of all bone indices by body part for a specific model
    pub fn map_bones_to_model(&self, bone_names: &[String]) -> HashMap<BodyPart, usize> {
        let mut result = HashMap::new();
        
        for body_part in [
            BodyPart::Head, BodyPart::Neck, 
            BodyPart::UpperSpine, BodyPart::MiddleSpine, BodyPart::LowerSpine, BodyPart::Pelvis,
            BodyPart::LeftShoulder, BodyPart::LeftElbow, BodyPart::LeftHand,
            BodyPart::RightShoulder, BodyPart::RightElbow, BodyPart::RightHand,
            BodyPart::LeftHip, BodyPart::LeftKnee, BodyPart::LeftFoot,
            BodyPart::RightHip, BodyPart::RightKnee, BodyPart::RightFoot,
        ] {
            if let Some(index) = self.find_bone_index(body_part, bone_names) {
                result.insert(body_part, index);
            }
        }
        
        result
    }
    
    /// Extract ordered vector of bone names from a model's bones
    pub fn extract_bone_names(bones: &[impl AsRef<str>]) -> Vec<String> {
        bones.iter()
            .map(|bone| bone.as_ref().to_string())
            .collect()
    }
}
