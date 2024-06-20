#![allow(non_camel_case_types)]

use mortalsim_macros::ParamEnum;

#[derive(Debug, Clone, Copy, ParamEnum)]
pub enum Smith2004CvsConstantParam {
    /// Resistance to flow into the left ventricle (mmHg_second_per_mL)
    R_mt,
    /// Resistance to flow out from the left ventricle (mmHg_second_per_mL)
    R_av,
    /// Resistance to flow into the right ventricle (mmHg_second_per_mL)
    R_tc,
    /// Resistance to flow out of the right ventricle (mmHg_second_per_mL)
    R_pv,
    /// Pulmonary circulation resistance (mmHg_second_per_mL)
    R_pul,
    /// Systolic circulation resistance (mmHg_second_per_mL)
    R_sys,
    /// Inertial divisor into the right ventricle (mmHg_second2_per_mL)
    L_tc,
    /// Inertial divisor out of the right ventricle (mmHg_second2_per_mL)
    L_pv,
    /// Inertial divisor into the left ventricle (mmHg_second2_per_mL)
    L_mt,
    /// Inertial divisor out of the left ventricle (mmHg_second2_per_mL)
    L_av,
    /// Thoracic pressure (mmHg)
    P_th,
    /// Driver function variable A (dimensionless)
    A,
    /// Driver function variable B (per_second_2)
    B,
    /// Driver function variable C (s)
    C,
    /// Pulse period (second)
    period,
    /// Initial pericardium pressure (mmHg)
    P_0_pcd,
    /// Initial pericardium volume (mL)
    V_0_pcd,
    /// Pericardium EDPVR fitting parameter (per_mL)
    lambda_pcd,
    /// Left ventricle free wall end systolic elastance (mmHg_per_mL)
    E_es_lvf,
    /// Left ventricle free wall volume at zero pressure (mL)
    V_d_lvf,
    /// Left ventricle free wall initial pressure (mmHg)
    P_0_lvf,
    /// Left ventricle EDPVR fitting parameter (per_mL)
    lambda_lvf,
    /// Left ventricle free wall initial volume (mmHg)
    V_0_lvf,
    /// Right ventricle free wall end systolic elastance (mmHg_per_mL)
    E_es_rvf,
    /// Right ventricle free wall volume at zero pressure (mL)
    V_d_rvf,
    /// Right ventricle free wall initial pressure (mmHg)
    P_0_rvf,
    /// Right ventricle EDPVR fitting parameter (per_mL)
    lambda_rvf,
    /// Right ventricle free wall initial volume (mL)
    V_0_rvf,
    /// Septum volume (mL)
    V_spt,
    /// Pulmonary artery end systolic elastance (mmHg_per_mL)
    E_es_pa,
    /// Pulmonary artery volume at zero pressure (mL)
    V_d_pa,
    /// Pulmonary vein end systolic elastance (mmHg_per_mL)
    E_es_pu,
    /// Pulmonary vein volume at zero pressure (mL)
    V_d_pu,
    /// Aorta end systolic elastance (mmHg_per_mL)
    E_es_ao,
    /// Aorta volume at zero pressure (mL)
    V_d_ao,
    /// Vena cava end systolic elastance (mmHg_per_mL)
    E_es_vc,
    /// Vena cava volume at zero pressure (mL)
    V_d_vc,
}

#[derive(Debug, Clone, Copy, ParamEnum)]
pub enum Smith2004CvsAssignmentParam {
    /// Time-varying elastance (dimensionless)
    e_t,
    /// Cardiac period time (second)
    tau,
    /// Pericardium free wall volume (mL)
    V_pcd,
    /// Pericardium free wall pressure (mmHg)
    P_pcd,
    /// Pericardium pressure (mmHg)
    P_peri,
    /// Left ventricle free wall volume (mL)
    V_lvf,
    /// Left ventricle free wall pressure (mmHg)
    P_lvf,
    /// Left ventricle pressure (mmHg)
    P_lv,
    /// Left ventricle free wall end systolic pressure (mmHg)
    P_es_lvf,
    /// Left ventricle free wall end diastolic pressure (mmHg)
    P_ed_lvf,
    /// Right ventricle free wall volume (mL)
    V_rvf,
    /// Right ventricle free wall pressure (mmHg)
    P_rvf,
    /// Right ventricle pressure (mmHg)
    P_rv,
    /// Right ventricle free wall end systolic pressure (mmHg)
    P_es_rvf,
    /// Right ventricle free wall end diastolic pressure (mmHg)
    P_ed_rvf,
    /// Pulmonary artery pressure (mmHg)
    P_pa,
    /// Pulmonary vein pressure (mmHg)
    P_pu,
    /// Aorta pressure (mmHg)
    P_ao,
    /// Vena cava pressure (mmHg)
    P_vc,
    /// Systolic flow rate (mL_per_second)
    Q_sys,
    /// Pulmonary flow rate (mL_per_second)
    Q_pul,
}

#[derive(Debug, Clone, Copy, ParamEnum)]
pub enum Smith2004CvsRateBoundParam {
    /// Left ventricle volume (mL)
    V_lv,
    /// Right ventricle volume (mL)
    V_rv,
    /// Pulmonary artery pressure (mL)
    V_pa,
    /// Pulmonary vein pressure (mL)
    V_pu,
    /// Volume entering aorta (mL)
    V_ao,
    /// Volume leaving vena cava (mL)
    V_vc,
    /// Flow rate into left ventricle (mL_per_second)
    Q_mt,
    /// Flow rate out of left ventricle (mL_per_second)
    Q_av,
    /// Flow rate into right ventricle (mL_per_second)
    Q_tc,
    /// Flow rate into left ventricle (mL_per_second)
    Q_pv,
}
