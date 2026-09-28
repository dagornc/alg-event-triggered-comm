# ALG_EVENT_TRIGGERED_COMM — portage Rust

Implémentation Rust de référence d'une politique de communication déclenchée par événement pour un essaim de 30 agents. Le crate et le binaire se nomment `event_triggered_comm_rs`.

## Ce qui est implémenté

Le simulateur discret maintient, pour chaque agent, un état local et le dernier état transmis. Une émission est déclenchée par priorité sur :

1. une alerte critique, qui contourne la période de garde ;
2. un franchissement de zone discrète ;
3. une excursion d'état atteignant le seuil configuré.

Le déclencheur d'état utilise deux seuils (seuil haut et seuil bas `seuil - hysteresis`) afin d'éviter les réactivations lorsque le signal oscille près de la frontière. Une période minimale de garde borne les émissions ordinaires. Quatre scénarios déterministes sont fournis : `nominal`, `turbulent`, `alerts` et `degraded`.

Les métriques CSV sont entièrement entières afin d'éviter une ambiguïté de comparaison : nombre de déclenchements par cause, nombre de messages périodiques de référence, réduction en points de base, débit moyen en milli-messages par pas, latence moyenne et maximale en microsecondes.

La référence Python normative est `reference/sim_event_triggered_comm.py`. Elle utilise `random.Random` de CPython ; le module Rust `src/rng.rs`, copié sans modification du socle RUST-13, reproduit ce générateur.

### Interface CLI commune

```text
event_triggered_comm_rs SEED AGENTS STEPS THRESHOLD HYSTERESIS GUARD_PERIOD HEARTBEAT_PERIOD SCENARIO
```

Exemple :

```sh
cargo run --release -- 1001 30 300 6 2 3 5 nominal
python3 reference/sim_event_triggered_comm.py 1001 30 300 6 2 3 5 nominal
```

Les deux commandes produisent le même en-tête et la même ligne CSV.

## Ce qui est vérifié

Vérifications réellement exécutées :

- `cargo build --release` : succès ;
- `cargo test` : **13 tests passés, 0 échec** (4 tests RNG, 2 tests CLI, 7 tests d'intégration) ;
- `python3 verify_parite_rust.py` : **200 comparaisons**, soit 50 graines × 4 scénarios ;
- résultat de parité : **200/200 sorties identiques, 0 écart — parité bit-à-bit vérifiée**.

Le harnais compare la ligne CSV complète, sans tolérance ni suppression de colonne.

Rejouer les contrôles :

```sh
cargo test
cargo build --release
python3 verify_parite_rust.py
```

Les invariants couvrent notamment le déterminisme, la réduction de charge nominale face au battement périodique, la priorité des alertes, la borne de garde, l'effet non croissant de l'hystérésis, l'augmentation de latence sur lien dégradé et l'horizon nul.

## Ce qui ne l'est pas

- Le modèle est une **implémentation conceptuelle de référence**, et non une validation sur radios, SITL/HITL ou essaim réel.
- La réduction spectrale et la borne de débit à 30 agents ne sont pas scientifiquement validées en vol ; elles sont uniquement mesurées dans ce simulateur discret.
- Les zones sont des bandes scalaires de largeur fixe, pas une géométrie 2D/3D ni des géofences opérationnelles.
- La latence est un modèle discret de service et de congestion, pas une caractérisation d'un modem réel ni du routage QAR.
- L'hypothèse du modèle LikeC4 est conservée : la sûreté locale et les alertes critiques ne dépendent pas du canal de coordination ordinaire.
- La parité vérifiée porte sur les deux implémentations de ce modèle ; elle ne constitue pas une validation scientifique de l'algorithme ou de ses paramètres.

Sources de conception : élément `algEventTriggeredComm` dans `e20-audit-completeness.c4`, avec les familles event-triggered consensus, QoS-aware routing et event-triggered allocation qui y sont référencées.

## Licence

MIT — Christophe Dagorn, 2026.
