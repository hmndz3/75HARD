//! Las reglas de "regañarme o felicitarme" (§6 del spec).
//!
//! Reglas duras, no IA: umbrales evaluados aquí, cero costo, cero latencia,
//! cero red. Funciones puras sobre los datos, con tests.
//!
//! Regla de diseño que no se negocia: el mensaje va SIEMPRE al hábito, nunca a
//! la persona. Ni siquiera en tono "duro". Un tracker que insulta se deja de
//! abrir en dos semanas — y peor, deja huella.

use serde::{Deserialize, Serialize};

use crate::daycut::format_minutes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tone {
    Suave,
    Directo,
    Duro,
}

impl Tone {
    pub fn from_setting(value: &str) -> Self {
        match value {
            "suave" => Tone::Suave,
            "duro" => Tone::Duro,
            _ => Tone::Directo,
        }
    }
}

/// Determina el color con que la UI pinta el mensaje. Nunca va color solo:
/// la UI acompaña siempre con icono o texto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Good,
    Neutral,
    Warning,
    Serious,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoachMessage {
    pub severity: Severity,
    pub text: String,
}

impl CoachMessage {
    fn new(severity: Severity, text: impl Into<String>) -> Self {
        Self {
            severity,
            text: text.into(),
        }
    }
}

/// Todo lo que el coach necesita saber del sueño de anoche.
#[derive(Debug, Clone, Copy)]
pub struct SleepFacts {
    pub minutes: i64,
    pub goal_min: i64,
    /// Noches seguidas por debajo de 6 h, contando la de anoche.
    pub short_nights_streak: u32,
}

const SHORT_NIGHT_MIN: i64 = 360; // 6 h

/// Veredicto inmediato del check-in matutino.
pub fn sleep_verdict(facts: SleepFacts, tone: Tone) -> CoachMessage {
    let slept = format_minutes(facts.minutes);

    // Un patrón pesa más que una noche suelta: se evalúa primero.
    if facts.short_nights_streak >= 3 && facts.minutes < SHORT_NIGHT_MIN {
        let n = facts.short_nights_streak;
        return CoachMessage::new(
            Severity::Serious,
            match tone {
                Tone::Suave => format!(
                    "{n} noches seguidas por debajo de 6 h. Vale la pena mirar qué está pasando con tus horarios."
                ),
                Tone::Directo => format!(
                    "{n}.ª noche seguida con poco sueño. Esto ya no es un mal día, es un patrón."
                ),
                Tone::Duro => format!(
                    "{n} noches seguidas bajo 6 h. El patrón está armado y lo estás sosteniendo tú."
                ),
            },
        );
    }

    if facts.minutes < SHORT_NIGHT_MIN {
        return CoachMessage::new(
            Severity::Critical,
            match tone {
                Tone::Suave => format!("{slept}. Es poco; intenta acostarte antes hoy."),
                Tone::Directo => format!("{slept}. Hoy vas a rendir menos y lo sabes."),
                Tone::Duro => {
                    format!("{slept}. El día empieza en déficit y eso se decidió anoche.")
                }
            },
        );
    }

    if facts.minutes >= facts.goal_min {
        return CoachMessage::new(
            Severity::Good,
            match tone {
                Tone::Suave => format!("{slept}. Buen descanso."),
                Tone::Directo => format!("{slept}. Así se ve un día bien empezado."),
                Tone::Duro => format!("{slept}. Cumpliste. Que no sea la excepción."),
            },
        );
    }

    // Entre 6 h y la meta.
    let falta = format_minutes(facts.goal_min - facts.minutes);
    CoachMessage::new(
        Severity::Warning,
        match tone {
            Tone::Suave => format!("{slept}. Te faltaron {falta} para tu meta."),
            Tone::Directo => format!("{slept}. {falta} por debajo de tu meta."),
            Tone::Duro => format!("{slept}. {falta} cortos otra vez."),
        },
    )
}

/// Estado del día para el panel de coach de la pantalla "Hoy".
#[derive(Debug, Clone)]
pub struct DayFacts {
    pub day_number: i64,
    pub target_days: i64,
    pub streak: i64,
    /// Pilares obligatorios que siguen sin cumplirse, en español y ya legibles.
    pub missing_pillars: Vec<String>,
    pub hours_left: i64,
    pub day_closed: bool,
}

/// Mensaje principal del panel de coach.
pub fn today_message(facts: &DayFacts, tone: Tone) -> CoachMessage {
    if facts.day_closed && facts.missing_pillars.is_empty() {
        return CoachMessage::new(
            Severity::Good,
            format!(
                "Día {} de {}. Racha intacta: {} días.",
                facts.day_number, facts.target_days, facts.streak
            ),
        );
    }

    if facts.missing_pillars.is_empty() {
        return CoachMessage::new(
            Severity::Good,
            format!(
                "Todos los pilares de hoy están cubiertos. Falta cerrar el día {}.",
                facts.day_number
            ),
        );
    }

    let lista = join_es(&facts.missing_pillars);
    let severity = if facts.hours_left <= 3 {
        Severity::Serious
    } else {
        Severity::Warning
    };

    CoachMessage::new(
        severity,
        match tone {
            Tone::Suave => format!("Te falta {lista}. Quedan {} horas.", facts.hours_left),
            Tone::Directo => format!(
                "Sin registrar: {lista}. Quedan {} horas del día {}.",
                facts.hours_left, facts.day_number
            ),
            Tone::Duro => format!(
                "{} horas y {lista} sigue en cero. El día {} se decide ahora.",
                facts.hours_left, facts.day_number
            ),
        },
    )
}

/// Pantalla de racha rota (P14). Aquí el tono baja: es un momento sensible.
pub fn streak_broken_message(day_number: i64, attempt: i64, reason: &str) -> CoachMessage {
    CoachMessage::new(
        Severity::Critical,
        format!(
            "Se rompió la racha en el día {day_number}. {reason} \
             Los datos de esos {day_number} días siguen aquí. Empezamos el intento #{attempt}."
        ),
    )
}

/// "a, b y c" — une una lista en español natural.
pub(crate) fn join_es(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} y {}", rest.join(", "), last),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(minutes: i64, streak: u32) -> SleepFacts {
        SleepFacts {
            minutes,
            goal_min: 420,
            short_nights_streak: streak,
        }
    }

    #[test]
    fn dormir_la_meta_felicita() {
        let m = sleep_verdict(facts(460, 0), Tone::Directo);
        assert_eq!(m.severity, Severity::Good);
        assert!(m.text.starts_with("7h 40min"));
    }

    #[test]
    fn dormir_poco_avisa_en_rojo() {
        let m = sleep_verdict(facts(320, 1), Tone::Directo);
        assert_eq!(m.severity, Severity::Critical);
        assert!(m.text.contains("5h 20min"));
    }

    #[test]
    fn entre_seis_horas_y_la_meta_es_ambar_y_dice_cuanto_falta() {
        let m = sleep_verdict(facts(400, 0), Tone::Directo);
        assert_eq!(m.severity, Severity::Warning);
        assert!(m.text.contains("20min por debajo"), "fue: {}", m.text);
    }

    #[test]
    fn tres_noches_cortas_seguidas_pesan_mas_que_una_sola() {
        let m = sleep_verdict(facts(330, 3), Tone::Directo);
        assert_eq!(m.severity, Severity::Serious);
        assert!(m.text.contains("patrón"));
    }

    #[test]
    fn el_patron_solo_aplica_si_anoche_tambien_fue_corta() {
        // Rompió la mala racha durmiendo bien: no se le recuerda el patrón.
        let m = sleep_verdict(facts(430, 3), Tone::Directo);
        assert_eq!(m.severity, Severity::Good);
    }

    #[test]
    fn los_tres_tonos_dan_texto_distinto_pero_misma_severidad() {
        let f = facts(320, 1);
        let a = sleep_verdict(f, Tone::Suave);
        let b = sleep_verdict(f, Tone::Directo);
        let c = sleep_verdict(f, Tone::Duro);
        assert_eq!(a.severity, c.severity);
        assert_ne!(a.text, b.text);
        assert_ne!(b.text, c.text);
    }

    #[test]
    fn ningun_tono_ataca_a_la_persona() {
        // Guardarraíl del §6: el regaño va al hábito.
        let prohibidas = ["eres", "inútil", "flojo", "fracas", "vergüenza"];
        for tone in [Tone::Suave, Tone::Directo, Tone::Duro] {
            for min in [200, 330, 400, 460] {
                for streak in [0, 3] {
                    let t = sleep_verdict(facts(min, streak), tone).text.to_lowercase();
                    for p in prohibidas {
                        assert!(!t.contains(p), "mensaje ofensivo: {t}");
                    }
                }
            }
        }
    }

    #[test]
    fn dia_con_pilares_pendientes_los_lista_en_espanol() {
        let f = DayFacts {
            day_number: 23,
            target_days: 75,
            streak: 22,
            missing_pillars: vec!["ejercicio".into(), "lectura".into(), "agua".into()],
            hours_left: 7,
            day_closed: false,
        };
        let m = today_message(&f, Tone::Directo);
        assert_eq!(m.severity, Severity::Warning);
        assert!(
            m.text.contains("ejercicio, lectura y agua"),
            "fue: {}",
            m.text
        );
    }

    #[test]
    fn con_pocas_horas_restantes_sube_la_severidad() {
        let f = DayFacts {
            day_number: 23,
            target_days: 75,
            streak: 22,
            missing_pillars: vec!["ejercicio".into()],
            hours_left: 2,
            day_closed: false,
        };
        assert_eq!(today_message(&f, Tone::Directo).severity, Severity::Serious);
    }

    #[test]
    fn dia_cerrado_y_completo_felicita_con_la_racha() {
        let f = DayFacts {
            day_number: 23,
            target_days: 75,
            streak: 23,
            missing_pillars: vec![],
            hours_left: 0,
            day_closed: true,
        };
        let m = today_message(&f, Tone::Duro);
        assert_eq!(m.severity, Severity::Good);
        assert!(m.text.contains("Día 23 de 75"));
    }

    #[test]
    fn racha_rota_recuerda_que_los_datos_siguen() {
        let m = streak_broken_message(31, 2, "Fallaste el pilar de ejercicio.");
        assert_eq!(m.severity, Severity::Critical);
        assert!(m.text.contains("siguen aquí"));
        assert!(m.text.contains("intento #2"));
    }
}
