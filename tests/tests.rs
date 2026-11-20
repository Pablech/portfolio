#[cfg(test)]
mod tests {
    use std::{num::NonZeroUsize, time::Duration};
    use lru::LruCache;
    use tokio::sync::Mutex;

    use site::repositories::games::{
            Agent, AgentKind, GameError, GameLogic, GameRoom, GameState, 
            MoveOutcome, PlayResult, RoomHandle, 
            connect4::Conn4Table, registry::build_agents, 
        };

    fn human_agents() -> Vec<Agent> {
        let mut agents = build_agents(Some("alice".to_string()), Some('X'), false, 2);

        agents[0].token = Some("tok-p1".into());
        agents[1].name = "bob".into();
        agents[1].token = Some("tok-p2".into());

        agents
    }

    fn human_room() -> Mutex<GameRoom<Conn4Table>> {
        Mutex::new(GameRoom {
            room_id: "testroom".into(),
            game: Conn4Table::new(),
            agents: human_agents(),
            turn: 0,
            finished: false,
            brains: vec![None, None],
            request_cache: LruCache::new(NonZeroUsize::new(10).unwrap()),
            rematch_votes: vec![false; 2],
            round: 0,
            last_chat: vec![None; 2],
            last_left: vec![None; 2],
            seat_grace: Duration::from_secs(3 * 60),
        })
    }

    struct AsymmetricMockGame {
    views: Vec<String>,
}

    impl GameLogic for AsymmetricMockGame {
        const SEATS: usize = 2;
        fn new() -> Self {
            Self {
                views: vec!["VISAO_SECRETA_P0".into(), "VISAO_SECRETA_P1".into()],
            }
        }
        fn legal_moves(&self) -> Vec<usize> {
            vec![0]
        }
        fn place(&mut self, _col: usize, _player: u8) -> Result<(), GameError> {
            Ok(())
        }
        fn check_win(&self, _player: u8) -> bool {
            false
        }
        fn check_draw(&self) -> bool {
            false
        }
        fn shared_output(&self) -> bool {
            false
        }
        fn output(&self, _agents: &[Agent], seat: &Option<usize>) -> String {
            match seat {
                Some(i) => self.views.get(*i).cloned().unwrap_or_default(),
                None => String::new(),
            }
        }
    }

    #[test]
    fn empate_total_sem_vencedor() {
        let mut t = Conn4Table::new();
        let piece = |r: usize, c: usize| -> u8 { if (r + 2 * c) % 4 < 2 { 1 } else { 2 } };

        for c in 0..7 {
            for r in (0..6).rev() {
                t.place(c, piece(r, c)).unwrap();
            }
        }

        assert!(t.check_draw(), "board cheio deve ser empate");
        assert!(!t.check_win(1) && !t.check_win(2), "pattern não pode dar 4 em linha");
        assert!(t.legal_moves().is_empty());
    }

    #[tokio::test]
    async fn vitoria_via_protocolo_e_empate_de_jogo() {
        let room = human_room();
        let seq = [
            ("tok-p1", "7", "bob"), ("tok-p2", "5", "alice"),
            ("tok-p1", "7", "bob"), ("tok-p2", "5", "alice"),
            ("tok-p1", "7", "bob"), ("tok-p2", "5", "alice"),
            ("tok-p1", "7", "WIN"),
        ];
        for (token, input, expect) in seq {
            let st = room.play(token, input, None).await.unwrap();
            match st {
                PlayResult::Fresh(GameState::State(MoveOutcome::Win { winner, .. })) => assert_eq!(winner, "alice"),
                PlayResult::Fresh(GameState::State(MoveOutcome::Continue { next_player, .. }) )=>
                    assert_eq!(next_player, expect),
                other => panic!("inesperado: {:?}", other),
            }
        }

        let err = room.play("tok-p1", "4", None).await.unwrap_err();
        assert!(matches!(err, GameError::ErrorMove(ref m) if m.contains("encerrada")));
    }

    #[test]
    fn coluna_cheia_e_rejeitada() {
        let mut t = Conn4Table::new();
        for _ in 0..6 {
            assert!(t.place(0, 1).is_ok());
        }
        assert!(matches!(t.place(0, 1), Err(GameError::ErrorMove(_))));
        assert!(!t.legal_moves().contains(&0));
    }

    #[test]
    fn is_win_with_detecta_vitoria_vertical_na_ultima_coluna() {
        let mut t = Conn4Table::new();
        for _ in 0..3 {
            t.place(6, 1).unwrap();
        }
        assert!(t.is_win_with(6, 1), "deveria detectar a quarta peça vencendo");
        assert!(!t.is_win_with(5, 1), "jogar noutra coluna não vence");
    }

    #[test]
    fn vitoria_horizontal_e_empate_na_regra() {
        let mut t = Conn4Table::new();
        t.place(0, 1).unwrap(); t.place(0, 2).unwrap();
        t.place(1, 1).unwrap(); t.place(1, 2).unwrap();
        t.place(2, 1).unwrap(); t.place(2, 2).unwrap();
        assert!(t.is_win_with(3, 1));
        t.place(3, 1).unwrap();
        assert!(t.check_win(1));
    }

    #[tokio::test]
    async fn vitoria_vertical_ponta_a_ponta_na_coluna_7() {
        let room = human_room();
        let seq = [
            ("tok-p1", "6", "bob"),
            ("tok-p2", "5", "alice"),
            ("tok-p1", "6", "bob"),
            ("tok-p2", "5", "alice"),
            ("tok-p1", "6", "bob"),
            ("tok-p2", "5", "alice"),
            ("tok-p1", "6", "WIN"),
        ];

        for (token, input, expect) in seq {
            let st = room.play(token, input, None).await.unwrap();
            match st {
                PlayResult::Fresh(GameState::State(MoveOutcome::Win { winner, .. }) )=> {
                    assert_eq!(winner, "alice");
                }
                PlayResult::Fresh(GameState::State(MoveOutcome::Continue { next_player, .. })) => {
                    assert_eq!(next_player, expect);
                }
                other => panic!("estado inesperado: {:?}", other),
            }
        }
    }

    #[tokio::test]
    async fn turno_certo_e_validacao_de_token() {
        let room = human_room();

        let err = room.play("tok-p2", "0", None).await.unwrap_err();
        assert!(matches!(err, GameError::ErrorMove(ref m) if m.contains("turno")));

        let err = room.play("token-falso", "0", None).await.unwrap_err();
        assert!(matches!(err, GameError::ErrorJoin(_)));

        let err = room.play("tok-p1", "banana", None).await.unwrap_err();
        assert!(matches!(err, GameError::ErrorMove(_)));

        let err = room.play("tok-p1", "99", None).await.unwrap_err();
        assert!(matches!(err, GameError::ErrorMove(_)));

        let info = room.join("tok-p2", None).await.unwrap();
        let GameState::JoinInfo { joined, room_id, .. } = info else {
            panic!("join deveria devolver JoinInfo");
        };
        assert_eq!(joined, "bob");
        assert_eq!(room_id, "testroom");
    }

    #[tokio::test]
    async fn partida_encerrada_rejeita_jogadas() {
        let room = human_room();
        for (token, input) in [
            ("tok-p1", "7"), ("tok-p2", "1"),
            ("tok-p1", "7"), ("tok-p2", "2"),
            ("tok-p1", "7"), ("tok-p2", "3"),
            ("tok-p1", "7"),
        ] {
            room.play(token, input, None).await.unwrap();
        }
        let err = room.play("tok-p1", "5", None).await.unwrap_err();
        assert!(matches!(err, GameError::ErrorMove(ref m) if m.contains("encerrada")));
    }

    #[test]
    fn build_agents_monta_o_que_se_espera() {

        let agents = build_agents(Some("alice".to_string()), Some('X'), true, 2);
        assert_eq!(agents.len(), 2);
        assert!(matches!(agents[0].kind, AgentKind::Human));
        assert!(agents[0].token.is_some(), "host sempre tem token");
        assert!(matches!(agents[1].kind, AgentKind::Cpu));
        assert!(agents[1].token.is_none(), "CPU não tem token (nunca teve)");
        assert_ne!(agents[1].fig, 'O', "CPU/pvp não pode herdar a figura do host");
        assert_eq!(agents[1].name, "Player 2", "assento pendente nasce com default posicional");

        let agents = build_agents(Some("outro".to_string()), Some('O'), false, 2);
        assert!(matches!(agents[1].kind, AgentKind::Human));
        assert!(agents[1].token.is_none(), "p2 nasce SEM token: assento vem do claim");
    }

    #[test]
    fn as_tuple_e_o_mapa_variante_codigo() {
        assert_eq!(GameError::ErrorMove("x".into()).as_tuple(), (409, "x"));
        assert_eq!(GameError::ErrorJoin("t".into()).as_tuple(), (403, "t"));
        assert_eq!(GameError::ErrorValidation("v".into()).as_tuple(), (422, "v"));
    }

    #[tokio::test]
    async fn test_idempotencia_request_cache() {
        let room = human_room();

        let st1 = match room.play("tok-p1", "7", Some("req-1")).await.unwrap() {
            PlayResult::Fresh(GameState::State(
                MoveOutcome::Continue { next_player, .. }
            )) => next_player,
            other => panic!("primeira deve ser Fresh+Continue: {:?}", other),
        };
        assert_eq!(st1, "bob");

        let st2 = match room.play("tok-p1", "7", Some("req-1")).await.unwrap() {
            PlayResult::Replayed(GameState::State(
                MoveOutcome::Continue { next_player, .. }
            )) => next_player,
            other => panic!("retry deve ser Replayed (ordem cache->turno?): {:?}", other),
        };
        assert_eq!(st2, "bob", "replay devolve o estado cacheado");

        let err = room.play("tok-p1", "4", Some("req-2")).await.unwrap_err();
        assert!(matches!(err, GameError::ErrorMove(ref m) if m.contains("turno")),
            "cache é por req_id, não global: {:?}", err);

        let _ = room.play("tok-p2", "banana", Some("req-3")).await.unwrap_err();
        let st3 = match room.play("tok-p2", "4", Some("req-3")).await.unwrap() {
            PlayResult::Fresh(GameState::State(
                MoveOutcome::Continue { next_player, .. }
            )) => next_player,
            other => panic!("req_id de erro não pode estar cacheado: {:?}", other),
        };
        assert_eq!(st3, "alice", "bob jogou de verdade; turno voltou pra alice");
    }

    #[test]
    fn for_seat_modo_compartilhado_entrega_uma_string_para_todos() {
        let st = GameState::State(MoveOutcome::Continue {
            output: vec!["TABULEIRO_UNICO".into()],
            next_player: "bob".into(),
        });

        let p0 = st.for_seat(0);
        let p1 = st.for_seat(1);

        match (p0, p1) {
            (
                GameState::State(MoveOutcome::Continue { output: o0, .. }),
                GameState::State(MoveOutcome::Continue { output: o1, .. }),
            ) => {
                assert_eq!(o0, vec!["TABULEIRO_UNICO"]);
                assert_eq!(o1, vec!["TABULEIRO_UNICO"]);
            }
            other => panic!("esperado Continue: {:?}", other),
        }
    }

    #[test]
    fn for_seat_modo_assimetrico_isola_visao_por_assento() {
        let st = GameState::State(MoveOutcome::Continue {
            output: vec!["SEGREDO_0".into(), "SEGREDO_1".into()],
            next_player: "alice".into(),
        });

        let GameState::State(MoveOutcome::Continue { output: o0, .. }) = st.for_seat(0) else {
            panic!("esperado Continue para seat 0");
        };
        let GameState::State(MoveOutcome::Continue { output: o1, .. }) = st.for_seat(1) else {
            panic!("esperado Continue para seat 1");
        };

        assert_eq!(o0, vec!["SEGREDO_0"]);
        assert_eq!(o1, vec!["SEGREDO_1"]);
        assert!(!o0[0].contains("SEGREDO_1"));
        assert!(!o1[0].contains("SEGREDO_0"));
    }

    #[tokio::test]
    async fn outputs_respeita_shared_output_false_na_sala() {
        let room = Mutex::new(GameRoom {
            room_id: "shiproom".into(),
            game: AsymmetricMockGame::new(),
            agents: human_agents(),
            turn: 0,
            finished: false,
            brains: vec![None, None],
            request_cache: LruCache::new(NonZeroUsize::new(10).unwrap()),
            rematch_votes: vec![false; 2],
            round: 0,
            last_chat: vec![None; 2],
            last_left: vec![None; 2],
            seat_grace: Duration::from_secs(180),
        });

        let st = room.snapshot().await;
        let GameState::State(MoveOutcome::Continue { output, .. }) = st.for_seat(1) else {
            panic!("esperado Continue");
        };
        assert_eq!(output, vec!["VISAO_SECRETA_P1"]);
    }

    #[tokio::test]
    async fn chat_valida_vazio_tamanho_e_rate_limit() {
        let room = human_room();

        let err_empty = room.chat("tok-p1", "   ").await.unwrap_err();
        assert!(matches!(err_empty, GameError::ErrorValidation(_)));

        let long_msg = "a".repeat(241);
        let err_long = room.chat("tok-p1", &long_msg).await.unwrap_err();
        assert!(matches!(err_long, GameError::ErrorValidation(_)));

        assert_eq!(room.chat("tok-p1", "oi").await.unwrap(), "alice");

        let err_rate = room.chat("tok-p1", "de novo").await.unwrap_err();
        assert!(matches!(err_rate, GameError::ErrorValidation(ref m) if m.contains("rate limit")));
    }
}