//! Tests d'intégration de la politique de communication événementielle.

use event_triggered_comm_rs::{Params, Scenario, simuler};

#[test]
fn nominal_reduit_la_charge_face_au_battement_periodique() {
    for seed in 1000..1050 {
        let r = simuler(seed, &Params::default());
        assert!(
            r.declenchements < r.messages_periodiques,
            "seed={seed}: {r:?}"
        );
    }
}

#[test]
fn les_alertes_critiques_declenchent_des_emissions() {
    let r = simuler(
        1001,
        &Params {
            scenario: Scenario::Alerts,
            ..Params::default()
        },
    );
    assert!(r.alertes > 0);
    assert!(r.declenchements >= r.alertes);
}

#[test]
fn l_hysteresis_reduit_ou_egale_les_declenchements() {
    for seed in 1000..1020 {
        let sans = simuler(
            seed,
            &Params {
                hysteresis: 0,
                ..Params::default()
            },
        );
        let avec = simuler(
            seed,
            &Params {
                hysteresis: 3,
                ..Params::default()
            },
        );
        assert!(
            avec.declenchements <= sans.declenchements,
            "seed={seed}: avec={} sans={}",
            avec.declenchements,
            sans.declenchements
        );
    }
}

#[test]
fn la_garde_limite_les_emissions_non_critiques() {
    let p = Params {
        scenario: Scenario::Turbulent,
        guard_period: 4,
        ..Params::default()
    };
    let r = simuler(1234, &p);
    let plafond = (p.steps as u64 * p.agents as u64).div_ceil(p.guard_period as u64);
    assert!(r.declenchements_etat + r.franchissements_zone <= plafond);
}

#[test]
fn le_lien_degrade_augmente_la_latence() {
    for seed in 1000..1020 {
        let nominal = simuler(seed, &Params::default());
        let degrade = simuler(
            seed,
            &Params {
                scenario: Scenario::Degraded,
                ..Params::default()
            },
        );
        assert!(
            degrade.latence_moyenne_us > nominal.latence_moyenne_us,
            "seed={seed}"
        );
    }
}

#[test]
fn aucune_emission_quand_horizon_nul() {
    let r = simuler(
        42,
        &Params {
            steps: 0,
            ..Params::default()
        },
    );
    assert_eq!(r.declenchements, 0);
    assert_eq!(r.debit_moyen_milli, 0);
    assert_eq!(r.latence_moyenne_us, 0);
}

#[test]
fn simulation_deterministe() {
    let p = Params {
        scenario: Scenario::Turbulent,
        ..Params::default()
    };
    assert_eq!(simuler(9999, &p), simuler(9999, &p));
}
