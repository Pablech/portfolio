#[cfg(test)]
mod tests {
    // use super::*;
    use tokio::sync::Mutex;

    use site::{
        repositories::games::{
            Agent, GameError, GameLogic, GameRoom, RoomHandle,
            ws::{GameState, MoveOutcome},
            connect4::Conn4Table,
            registry::build_agents,
            structs::AgentKind,
            
        }
    };

    fn human_agents() -> Vec<Agent> {
        vec![
            Agent {
                kind: AgentKind::Human,
                name: "alice".into(),
                fig: '1'.into(),
                token: Some("tok-p1".into()),
            },
            Agent {
                kind: AgentKind::Human,
                name: "bob".into(),
                fig: '2'.into(),
                token: Some("tok-p2".into()),
            },
        ]
    }

    fn human_room() -> Mutex<GameRoom<Conn4Table>> {
        Mutex::new(GameRoom {
            room_id: "testroom".into(),
            game: Conn4Table::new(),
            agents: human_agents(),
            turn: 0,
            finished: false,
            brains: vec![None, None],
        })
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
            let st = room.play(token, input).await.unwrap();
            match st {
                GameState::State(MoveOutcome::Win { winner, .. }) => assert_eq!(winner, "alice"),
                GameState::State(MoveOutcome::Continue { next_player, .. }) =>
                    assert_eq!(next_player, expect),
                other => panic!("inesperado: {:?}", other),
            }
        }

        let err = room.play("tok-p1", "4").await.unwrap_err();
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
            let st = room.play(token, input).await.unwrap();
            match st {
                GameState::State(MoveOutcome::Win { winner, .. }) => {
                    assert_eq!(winner, "alice");
                }
                GameState::State(MoveOutcome::Continue { next_player, .. }) => {
                    assert_eq!(next_player, expect);
                }
                other => panic!("estado inesperado: {:?}", other),
            }
        }
    }

    #[tokio::test]
    async fn turno_certo_e_validacao_de_token() {
        let room = human_room();

        let err = room.play("tok-p2", "0").await.unwrap_err();
        assert!(matches!(err, GameError::ErrorMove(ref m) if m.contains("turno")));

        let err = room.play("token-falso", "0").await.unwrap_err();
        assert!(matches!(err, GameError::ErrorJoin(_)));

        let err = room.play("tok-p1", "banana").await.unwrap_err();
        assert!(matches!(err, GameError::ErrorMove(_)));

        let err = room.play("tok-p1", "99").await.unwrap_err();
        assert!(matches!(err, GameError::ErrorMove(_)));

        let info = room.join("tok-p2").await.unwrap();
        let GameState::JoinInfo { joined, room_id } = info else {
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
            room.play(token, input).await.unwrap();
        }
        let err = room.play("tok-p1", "5").await.unwrap_err();
        assert!(matches!(err, GameError::ErrorMove(ref m) if m.contains("encerrada")));
    }

    #[test]
    fn build_agents_monta_o_que_se_espera() {
        let vs_cpu = build_agents("a".into(), 'X', true);
        assert_eq!(vs_cpu.len(), 2);
        assert!(vs_cpu[0].token.is_some());
        assert!(vs_cpu[1].token.is_none(), "CPU não porta token");
        assert_eq!(vs_cpu[1].name, "Cpu.ai");

        let pvp = build_agents("a".into(), 'X', false);
        assert!(pvp.iter().all(|a| a.token.is_some()), "pvp: ambos com token");
    }
}