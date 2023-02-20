//! # Cactuar!
//!
//! ```text
//!              BG
//!         BB    PB
//!          BP&   P#
//!     BG#   #PB&#PY5555PGB&
//!      &BG#  GJ??J???JYJ??JYG&
//!         &GJ77J?7JYYJJ55Y7?YJG
//!         #!!7!?YY??Y55YY5PY?YY5&
//!         J~!?7~?Y5YJJ5P5Y5P5?J5YG
//!         ?~~7J7!7YPPYJ5PPPBBGY?YJ5&
//!         5^~~7JJ!7J5P5YY5&   GPJJ5YB              &BGPPPB
//!          J~!!7JY?7Y55PYJ5GBG5PG5JYY5&        &#GPYYYYYYYYB
//!           P77!!?5# &PYGPYY5P5Y5GGJJ5YB    &BP55YY55PP5555Y5&
//!            #J!7!?G&&5?5PP55PPP5YPG5JY5P##G5YY555PPGGPP5PP55YB
//!              G7777JYY??Y5B  BPGP55GGYYP55555PPPGGBB#BP555PP5YP&
//!               &Y777?Y5J?J5B  #PPP55PGP5GGGGGGBB#&    &G555PPPP5G
//!                 GJ???J5Y??YP&  BPGGPPGBBBBB#&&        &BP555PPP5P&
//!  #GPPGB&         &PJJ?JYYJ?J5B&#5PPGGGPBB####&          #G555PPGG#
//! 5!?55PPG#          BYJJ?JYY??YY5G5Y5PBBPGB##BG#          &BPPPGB#&
//! 7~7J5GGGGB&         &PYYJJY5J?J5PP5Y5PBBGGB##BGG&          &&&&
//! &J77?5PGGGG&      #G55YYYJ?JYYJJY5GPYYPGBBGGBBGGPB
//!   G?7?J5PGGG#&&#G5YJY5555YJJ?Y5J?J5PG5Y5PBBGGGBBGGG#
//!    &577?YPPPGP5YYY55PPPPPGYJJ?JYY??YPGPYYPGBBGPB#BGGB
//!      B?7?J5P55YY5PPPPGG#& &P????YYJ7J5PP5Y5PB#GGB##BPP&
//!       &57??JY55PPGGB#&      #J???JYJ7?Y5PPY5GB#BGB##GGPB               &&#BB#&
//!         #5JJ5PPGB#&           P?7??JY??Y5PPPPGB##BB#BGGGP#          &BGP555YJYG
//!           &B#&&                #J???JYYJ5GGBGGB####BGGGPGPB      &BG55YYYY5PGBBB
//!                                 57?JJY5PPPBBBBBB#&#PPPPPPGGB#  #G5555YYY55PGBB##
//!                              #G5YYYYJY5PGGGB###&##GGPP5PGGGGPGGP555555PPGGBBGB&
//!                           &BP5YYYY5555PGBBBB#&&  &#BGPPPPGBBGP55555PGGGBBBB#&
//!                         #P5YYY5555PPPGGGGB#&       &BGPPPPPBGPPPPPGGGGBB#&
//!                       GYYYYYY5PPPGGGGGBB&            #BGPP5PGGGGGGGGB#&
//!                      P~?YJY5PGBBGGGB#&                &#GP5PPGGGGB#&
//!                      G~7J?5PGB#GG#&                     &#BBB###&
//!                       G7J?J5PGBBGBB#
//!                        #Y???YPPGBGPGB&
//!                          G???J5PGGGGGGB
//!                           &5?J?Y5PPGPGPG&
//!                             GJJJJY5PGGGGG&
//!                              &5J??Y5PGGG##
//!                                GYJJYPPG##
//!                                 &GPPGB#&
//! ```
//!
//! Kubernetes controller for creating Prometheus alerts using standard metrics
//! emitted by an Istio sidecar container.

pub mod config;
pub mod controller;
pub mod http;
pub mod logging;
pub mod prometheus;
pub mod service_alerts;
