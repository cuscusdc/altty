# Código integral do port kazzu.html

Os arquivos abaixo correspondem ao aplicativo em crates/kazzu-html-player.

## Árvore

```text
Cargo.toml
build.rs
ui/theme.slint
ui/components/action.slint
ui/components/chips.slint
ui/components/confirmation.slint
ui/components/field.slint
ui/components/glass.slint
ui/components/heading.slint
ui/components/island.slint
ui/components/queue.slint
ui/components/scrubber.slint
ui/components/toast.slint
ui/components/track-card.slint
ui/components/track-row.slint
ui/screens/biblioteca.slint
ui/screens/configuracoes.slint
ui/screens/explorar.slint
ui/screens/inicio.slint
ui/screens/pesquisa.slint
ui/screens/player.slint
ui/app.slint
src/models.rs
src/audio.rs
src/main.rs
ui/state.slint
ui/icons.slint
src/snapshot.rs
src/palettes.json
src/settings.json
src/tracks.json
ui/icons/collapse.svg
ui/icons/heart.svg
ui/icons/history.svg
ui/icons/lyrics.svg
ui/icons/more.svg
ui/icons/next.svg
ui/icons/pause.svg
ui/icons/play.svg
ui/icons/previous.svg
ui/icons/queue.svg
ui/icons/repeat.svg
ui/icons/search.svg
ui/icons/settings.svg
ui/icons/share.svg
ui/icons/shuffle.svg
ui/icons/timer.svg
```

## Cargo.toml

```toml
[package]
name = "kazzu-html-player"
version = "0.1.0"
edition = "2024"
build = "build.rs"

[dependencies]
slint = { version = "1.19", default-features = false, features = ["std", "backend-default", "renderer-software", "renderer-femtovg", "compat-1-2", "accessibility"] }
arboard = { version = "3.7", default-features = false }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[build-dependencies]
slint-build = "1.19"
```

## build.rs

```rust
fn main() {
    slint_build::compile("ui/app.slint").expect("compilar interface kazzu");
}
```

## ui/theme.slint

```slint
export global Theme {
    in-out property <color> accent: #ff6b00;
    in-out property <color> secondary: #ffa858;
    in-out property <color> base: #120a05;
    in-out property <color> card: #1e120ab8;
    in-out property <bool> glass: true;
    in-out property <bool> reduce-motion: true;
    out property <color> ink: #fdf6ec;
    out property <color> muted: #fdf6ecc2;
    out property <color> tertiary: #fdf6ec94;
    out property <length> card-radius: 18px;
    out property <length> tile-radius: 16px;
    out property <length> inner-radius: 12px;
}
```

## ui/components/action.slint

```slint
import { Theme } from "../theme.slint";
export component Action inherits Rectangle {
    in property <string> text;
    in property <image> icon;
    in property <length> font-size: 12px;
    in property <int> font-weight: 600;
    in property <bool> active;
    in property <bool> filled;
    in property <bool> bright;
    callback clicked();
    min-width: 44px; min-height: 44px;
    border-radius: 16px;
    background: root.bright ? #ffffff : root.active ? Theme.accent : root.filled ? #ffffff14 : touch.pressed ? #ffffff22 : transparent;
    accessible-role: button; accessible-label: root.text;
    if root.icon.width > 0 : Image { source: root.icon; width: 22px; height: 22px; x: (parent.width - self.width) / 2; y: (parent.height - self.height) / 2; colorize: root.bright ? #12101b : Theme.ink; }
    if root.icon.width == 0 : Text { text: root.text; color: root.bright ? #12101b : Theme.ink; font-size: root.font-size; font-weight: root.font-weight; horizontal-alignment: center; vertical-alignment: center; overflow: elide; }
    touch := TouchArea { clicked => { root.clicked(); } }
    FocusScope { key-pressed(e) => { if (e.text == " " || e.text == "\n") { root.clicked(); return accept; } return reject; } }
}
```

## ui/components/chips.slint

```slint
import { Action } from "action.slint";
export component Chips inherits Flickable {
    in property <[string]> options;
    in-out property <int> selected;
    height: 44px; content-height: self.height; content-width: root.options.length * 118px;
    line := HorizontalLayout { width: root.content-width; spacing: 8px;
        for text[i] in root.options : Action { width: 110px; text: text; active: root.selected == i; filled: true; clicked => { root.selected = i; } }
    }
}
```

## ui/components/confirmation.slint

```slint
import { State } from "../state.slint";
import { Glass } from "glass.slint";
import { Action } from "action.slint";
import { Theme } from "../theme.slint";
export component Confirmation inherits Rectangle {
    background: #000000aa;
    TouchArea { }
    Glass { x: 16px; width: parent.width - 32px; y: (parent.height - self.height) / 2; height: 200px; background: Theme.base;
        VerticalLayout { padding: 18px; spacing: 12px;
            Text { text: State.confirm == 1 ? "Deseja realmente limpar o cache de downloads offline (2.4 GB)? As músicas continuarão disponíveis para streaming." : "Limpar todo o histórico recente de pesquisas do kazzu?"; color: Theme.ink; font-size: 14px; wrap: word-wrap; }
            HorizontalLayout { Action { text: "Cancelar"; clicked => { State.confirm = 0; } } Action { text: "Limpar"; active: true; clicked => { State.command(State.confirm == 1 ? "clear-cache" : "clear-history"); State.confirm = 0; } } }
        }
    }
}
```

## ui/components/field.slint

```slint
import { Theme } from "../theme.slint";
export component Field inherits Rectangle {
    in-out property <string> text;
    in property <string> placeholder-text;
    callback edited(string);
    min-height: 44px; border-radius: 24px; background: #fdf6ec14;
    border-width: 1px; border-color: Theme.accent.with-alpha(0.28);
    TouchArea { clicked => { input.focus(); } }
    if root.text == "" : Text { x: 16px; width: parent.width - 32px; text: root.placeholder-text; color: Theme.muted; font-size: 14px; vertical-alignment: center; }
    input := TextInput { x: 16px; width: parent.width - 32px; height: parent.height; text <=> root.text; color: Theme.ink; font-size: 14px; vertical-alignment: center; single-line: true; edited => { root.edited(self.text); } }
}
```

## ui/components/glass.slint

```slint
import { Theme } from "../theme.slint";
export component Glass inherits Rectangle {
    border-radius: Theme.card-radius;
    background: Theme.glass ? @linear-gradient(135deg, #ffffff29, #ffffff0a 45%, #ffffff14) : Theme.card;
    border-width: 1px;
    border-color: #ffffff33;
    drop-shadow-color: #00000061;
    drop-shadow-blur: 20px;
    drop-shadow-offset-y: 8px;
    Rectangle { x: 12px; y: 1px; width: max(0px, parent.width - 24px); height: 1px; background: #ffffff4d; }
    @children
}
```

## ui/components/heading.slint

```slint
import { Theme } from "../theme.slint";
export component Heading inherits Text { height: 32px; color: Theme.ink; font-size: 16px; font-weight: 800; vertical-alignment: center; }
```

## ui/components/island.slint

```slint
import { Icons } from "../icons.slint";
import { State } from "../state.slint";
import { Theme } from "../theme.slint";
import { Glass } from "glass.slint";
import { Action } from "action.slint";
export component Island inherits Rectangle {
    height: 124px;
    Glass { height: 58px; width: parent.width; border-radius: 22px;
        HorizontalLayout { padding-left: 12px; padding-right: 12px; spacing: 4px;
            Rectangle { horizontal-stretch: 1;
                HorizontalLayout { spacing: 12px; alignment: center;
                    Rectangle { width: 42px; height: 42px; border-radius: 12px; background: @linear-gradient(135deg, State.songs[State.current].first, State.songs[State.current].last); Text { text: "♫"; color: white; horizontal-alignment: center; vertical-alignment: center; } }
                    VerticalLayout { alignment: center;
                        Text { text: State.songs[State.current].title; color: Theme.ink; font-size: 14px; font-weight: 800; overflow: elide; }
                        Text { text: State.songs[State.current].artist; color: Theme.muted; font-size: 12px; overflow: elide; }
                    }
                }
                TouchArea { clicked => { State.expanded = true; } }
            }
            Action { width: 44px; text: State.playing ? "Pausar" : "Reproduzir"; icon: State.playing ? Icons.pause : Icons.play; clicked => { State.command("play"); } }
            Action { width: 44px; text: "Próxima faixa"; icon: Icons.next; clicked => { State.command("next"); } }
        }
        Rectangle { y: parent.height - 3px; height: 3px; width: parent.width * State.progress; background: Theme.accent; }
    }
    HorizontalLayout { y: 66px; height: 58px; width: parent.width; spacing: 10px;
        Glass { border-radius: 22px; HorizontalLayout { padding-left: 12px; padding-right: 12px;
            for title[i] in ["Início", "Explorar", "Biblioteca"] : Action { text: title; active: State.page == i; clicked => { State.go(i); } }
        } }
        Action { width: 58px; height: 58px; border-radius: 22px; text: "Pesquisar"; icon: Icons.search; active: State.page == 3; filled: true; clicked => { State.go(3); } }
    }
}
```

## ui/components/queue.slint

```slint
import { State } from "../state.slint";
import { Action } from "action.slint";
import { TrackRow } from "track-row.slint";
import { Glass } from "glass.slint";
export component Queue inherits Glass {
    background: #14111d; border-radius: 24px;
    VerticalLayout { padding: 16px; spacing: 6px;
        Action { text: "Fila de reprodução   ×"; clicked => { State.queue = false; } }
        Flickable { content-width: self.width; VerticalLayout { width: parent.width; spacing: 6px;
            for song[i] in State.songs : TrackRow { song: song; current: State.current == i; clicked => { State.select(i, true); State.queue = false; } }
        } }
    }
}
```

## ui/components/scrubber.slint

```slint
import { Theme } from "../theme.slint";
export component Scrubber inherits Rectangle {
    in property <float> value;
    in property <string> label;
    in property <length> thickness: 6px;
    callback changed(float);
    height: 44px; min-width: 44px;
    accessible-role: slider;
    accessible-label: root.label;
    accessible-value: root.value * 100;
    Rectangle { y: (parent.height - self.height) / 2; height: root.thickness; width: parent.width; border-radius: 999px; background: #ffffff33;
        Rectangle { width: parent.width * clamp(root.value, 0, 1); border-radius: 999px; background: Theme.secondary; }
    }
    Rectangle { x: clamp(root.value, 0, 1) * (parent.width - 14px); y: (parent.height - 14px) / 2; width: 14px; height: 14px; border-radius: 7px; background: white; }
    touch := TouchArea {
        pointer-event(event) => { if (event.kind == PointerEventKind.down || event.kind == PointerEventKind.up) { root.changed(clamp(self.mouse-x / self.width, 0, 1)); } }
        moved => { if (self.pressed) { root.changed(clamp(self.mouse-x / self.width, 0, 1)); } }
    }
    FocusScope { key-pressed(event) => {
        if (event.text == Key.LeftArrow) { root.changed(max(0, root.value - 0.01)); return accept; }
        if (event.text == Key.RightArrow) { root.changed(min(1, root.value + 0.01)); return accept; }
        return reject;
    } }
}
```

## ui/components/toast.slint

```slint
import { Glass } from "glass.slint";
import { Theme } from "../theme.slint";
export component Toast inherits Glass { in property <string> message; height: 64px; border-radius: 22px; Text { x: 14px; width: parent.width - 28px; text: root.message; color: Theme.ink; font-size: 13px; wrap: word-wrap; vertical-alignment: center; } }
```

## ui/components/track-card.slint

```slint
import { Theme } from "../theme.slint";
import { Glass } from "glass.slint";
import { Song } from "../state.slint";
export component TrackCard inherits Glass {
    in property <Song> song;
    in property <bool> random;
    callback clicked();
    height: self.width * 1.05;
    border-radius: Theme.tile-radius;
    VerticalLayout {
        padding: 8px; spacing: 4px;
        Rectangle { border-radius: 12px; background: @linear-gradient(135deg, root.song.first, root.song.last); vertical-stretch: 1;
            Text { text: root.random ? "⚄" : "♫"; color: Theme.ink; font-size: 24px; horizontal-alignment: center; vertical-alignment: center; }
        }
        Text { text: root.random ? "Aleatório" : root.song.title; color: Theme.ink; font-size: 11px; font-weight: 700; overflow: elide; }
        Text { text: root.random ? "Surpreenda-me" : root.song.artist; color: Theme.muted; font-size: 11px; overflow: elide; }
    }
    TouchArea { clicked => { root.clicked(); } }
}
```

## ui/components/track-row.slint

```slint
import { Theme } from "../theme.slint";
import { Glass } from "glass.slint";
import { Song } from "../state.slint";
export component TrackRow inherits Glass {
    in property <Song> song;
    in property <bool> current;
    callback clicked();
    height: 60px; border-radius: 14px;
    border-color: root.current ? Theme.accent : #ffffff20;
    HorizontalLayout {
        padding: 10px; spacing: 12px;
        Rectangle { width: 38px; border-radius: 10px; background: @linear-gradient(135deg, root.song.first, root.song.last); }
        VerticalLayout { alignment: center;
            Text { text: root.song.title; color: Theme.ink; font-size: 13px; font-weight: 700; overflow: elide; }
            Text { text: root.song.artist; color: Theme.muted; font-size: 11px; overflow: elide; }
        }
        Text { text: root.song.duration; color: Theme.muted; font-size: 11px; vertical-alignment: center; }
    }
    TouchArea { clicked => { root.clicked(); } }
}
```

## ui/screens/biblioteca.slint

```slint
import { Theme } from "../theme.slint";
import { State } from "../state.slint";
import { Glass } from "../components/glass.slint";
import { Action } from "../components/action.slint";
import { Heading } from "../components/heading.slint";
import { Chips } from "../components/chips.slint";
export component Biblioteca inherits VerticalLayout {
    spacing: 12px;
    Action { text: "+ Nova playlist"; clicked => { State.command("playlist"); } }
    Chips { options: ["Playlists", "Faixas", "Artistas", "Álbuns"]; }
    Glass { height: 130px;
        VerticalLayout { padding: 16px; spacing: 8px;
            Text { text: "SUA EXPERIÊNCIA DE ESCUTA"; color: Theme.secondary; font-size: 11px; }
            Text { text: "Membro desde 10/26"; color: Theme.muted; font-size: 11px; }
            Text { text: "694 MINUTOS OUVIDOS"; color: Theme.ink; font-size: 24px; font-weight: 900; }
            Text { text: "• cuscusdoce rael · 170 reproduções totais"; color: Theme.muted; font-size: 11px; wrap: word-wrap; }
        }
    }
    Heading { text: "Coleções"; }
    HorizontalLayout { spacing: 9px;
        for title in ["♥ Curtidas", "↓ Baixados", "↗ Top 100"] : Action { height: 104px; text: title; filled: true; clicked => { State.command(title); } }
    }
}
```

## ui/screens/configuracoes.slint

```slint
import { State } from "../state.slint";
import { Theme } from "../theme.slint";
import { Glass } from "../components/glass.slint";
import { Action } from "../components/action.slint";
import { Heading } from "../components/heading.slint";
import { Field } from "../components/field.slint";
export component Configuracoes inherits VerticalLayout {
    spacing: 12px;
    Field { height: 44px; placeholder-text: "Pesquisar ajustes..."; text <=> State.settings-query; edited(q) => { State.filter-settings(q); } }
    Action { height: 74px; text: "C   cuscusdoce rael · Membro Free • Gerenciar conta"; filled: true; clicked => { State.command("profile"); } }
    Heading { text: "Paletas Temáticas do App"; }
    for row in 2 : HorizontalLayout { spacing: 8px;
        for column in 3 : Action { height: 66px; text: ["Laranja Solar", "Rosa Cyber", "Ametista", "Esmeralda", "Azul Cobalto", "Obsidian Ouro"][row * 3 + column]; active: State.palette == row * 3 + column; filled: true; clicked => { State.theme(row * 3 + column); } }
    }
    for setting[i] in State.settings : VerticalLayout { spacing: 6px;
        if i == 0 || setting.group != State.settings[i - 1].group : Heading { text: setting.group; font-size: 12px; color: Theme.secondary; }
        Glass { height: 78px;
            HorizontalLayout { padding: 12px; spacing: 8px;
                VerticalLayout { alignment: center;
                    Text { text: setting.title; font-size: 14px; color: Theme.ink; font-weight: 700; overflow: elide; }
                    Text { text: setting.subtitle; font-size: 12px; color: Theme.muted; overflow: elide; }
                }
                Action { width: 64px; text: setting.toggle ? (setting.checked ? "Ligado" : "Desligado") : setting.value != "" ? setting.value : "›"; active: setting.toggle && setting.checked; clicked => { State.setting(i); } }
            }
        }
    }
}
```

## ui/screens/explorar.slint

```slint
import { State } from "../state.slint";
export component Explorar inherits VerticalLayout { spacing: 12px;
HorizontalLayout { spacing: 12px; Rectangle { height: 94px; border-radius: 18px; background: #55b768; VerticalLayout { padding: 12px; Text { text: "Clima alegre"; color: white; font-size: 14px; font-weight: 700; wrap: word-wrap; } Text { text: "Sunshine Indie"; color: white; font-size: 12px; horizontal-alignment: right; wrap: word-wrap; } } TouchArea { clicked => { State.select(4, false); State.expanded = true; } } }Rectangle { height: 94px; border-radius: 18px; background: #d79700; VerticalLayout { padding: 12px; Text { text: "Em trânsito"; color: white; font-size: 14px; font-weight: 700; wrap: word-wrap; } Text { text: "Feel-Good Hip-Hop and R&B"; color: white; font-size: 12px; horizontal-alignment: right; wrap: word-wrap; } } TouchArea { clicked => { State.select(5, false); State.expanded = true; } } } }
HorizontalLayout { spacing: 12px; Rectangle { height: 94px; border-radius: 18px; background: #d3be62; VerticalLayout { padding: 12px; Text { text: "Energia"; color: white; font-size: 14px; font-weight: 700; wrap: word-wrap; } Text { text: "Pop de bombeamento"; color: white; font-size: 12px; horizontal-alignment: right; wrap: word-wrap; } } TouchArea { clicked => { State.select(6, false); State.expanded = true; } } }Rectangle { height: 94px; border-radius: 18px; background: #7c58d8; VerticalLayout { padding: 12px; Text { text: "Festa"; color: white; font-size: 14px; font-weight: 700; wrap: word-wrap; } Text { text: "Festa Sertaneja"; color: white; font-size: 12px; horizontal-alignment: right; wrap: word-wrap; } } TouchArea { clicked => { State.select(7, false); State.expanded = true; } } } }
HorizontalLayout { spacing: 12px; Rectangle { height: 94px; border-radius: 18px; background: #888d95; VerticalLayout { padding: 12px; Text { text: "Foco"; color: white; font-size: 14px; font-weight: 700; wrap: word-wrap; } Text { text: "Lofi Loft"; color: white; font-size: 12px; horizontal-alignment: right; wrap: word-wrap; } } TouchArea { clicked => { State.select(1, false); State.expanded = true; } } }Rectangle { height: 94px; border-radius: 18px; background: #4a4d52; VerticalLayout { padding: 12px; Text { text: "Games"; color: white; font-size: 14px; font-weight: 700; wrap: word-wrap; } Text { text: "GAMING HITS"; color: white; font-size: 12px; horizontal-alignment: right; wrap: word-wrap; } } TouchArea { clicked => { State.select(7, false); State.expanded = true; } } } }
HorizontalLayout { spacing: 12px; Rectangle { height: 94px; border-radius: 18px; background: #2762c4; VerticalLayout { padding: 12px; Text { text: "Hanukkah"; color: white; font-size: 14px; font-weight: 700; wrap: word-wrap; } Text { text: ""; color: white; font-size: 12px; horizontal-alignment: right; wrap: word-wrap; } } TouchArea { clicked => { State.select(0, false); State.expanded = true; } } }Rectangle { height: 94px; border-radius: 18px; background: #6c99db; VerticalLayout { padding: 12px; Text { text: "Músicas para relaxar"; color: white; font-size: 14px; font-weight: 700; wrap: word-wrap; } Text { text: "Coffee Shop Blend"; color: white; font-size: 12px; horizontal-alignment: right; wrap: word-wrap; } } TouchArea { clicked => { State.select(3, false); State.expanded = true; } } } }
HorizontalLayout { spacing: 12px; Rectangle { height: 94px; border-radius: 18px; background: #138535; VerticalLayout { padding: 12px; Text { text: "Natal"; color: white; font-size: 14px; font-weight: 700; wrap: word-wrap; } Text { text: "Funk de Natal"; color: white; font-size: 12px; horizontal-alignment: right; wrap: word-wrap; } } TouchArea { clicked => { State.select(2, false); State.expanded = true; } } }Rectangle { height: 94px; border-radius: 18px; background: #b01a1a; VerticalLayout { padding: 12px; Text { text: "Romance"; color: white; font-size: 14px; font-weight: 700; wrap: word-wrap; } Text { text: "Pagode & Sertanejo"; color: white; font-size: 12px; horizontal-alignment: right; wrap: word-wrap; } } TouchArea { clicked => { State.select(0, false); State.expanded = true; } } } }
}
```

## ui/screens/inicio.slint

```slint
import { State } from "../state.slint";
import { TrackCard } from "../components/track-card.slint";
import { Heading } from "../components/heading.slint";
import { Chips } from "../components/chips.slint";
export component Inicio inherits VerticalLayout {
    spacing: 12px;
    Chips { options: ["Todos", "Podcasts", "Energia", "Romance", "Para o trânsito"]; }
    Heading { text: "⚡ Escolha rápida"; }
    for row in 3 : HorizontalLayout { spacing: 9px;
        for column in 3 : TrackCard { horizontal-stretch: 1; song: State.songs[mod(row * 3 + column + 5, 8)]; random: row == 2 && column == 2;
            clicked => { if (row == 2 && column == 2) { State.command("random"); } else { State.select(mod(row * 3 + column + 5, 8), false); State.expanded = true; } }
        }
    }
    Heading { text: State.continue-title; }
    Flickable { height: 206px; content-height: self.height; content-width: 459px;
        rail := HorizontalLayout { width: 459px; spacing: 12px;
            for index in State.continue-tracks : TrackCard { width: 145px; height: 190px; song: State.songs[index]; clicked => { State.select(index, false); State.expanded = true; } }
        }
    }
}
```

## ui/screens/pesquisa.slint

```slint
import { State } from "../state.slint";
import { Field } from "../components/field.slint";
import { Action } from "../components/action.slint";
import { Heading } from "../components/heading.slint";
import { TrackRow } from "../components/track-row.slint";
import { Chips } from "../components/chips.slint";
export component Pesquisa inherits VerticalLayout {
    spacing: 8px;
    Field { height: 48px; placeholder-text: "Artistas, músicas ou letras..."; text <=> State.query; edited(text) => { State.search(text); } }
    Chips { options: ["YouTube", "Biblioteca"]; }
    HorizontalLayout { Heading { text: "Pesquisas recentes"; } Action { text: "Limpar"; clicked => { State.confirm = 2; } } }
    if State.history && State.query == "" : VerticalLayout {
        for i in [0, 1] : TrackRow { song: State.songs[i]; clicked => { State.select(i, false); State.expanded = true; } }
    }
    for song in State.results : TrackRow { song: song; clicked => { State.command("find:" + song.title); State.expanded = true; } }
}
```

## ui/screens/player.slint

```slint
import { Icons } from "../icons.slint";
import { State } from "../state.slint";
import { Theme } from "../theme.slint";
import { Action } from "../components/action.slint";
import { Glass } from "../components/glass.slint";
import { Scrubber } from "../components/scrubber.slint";
export component Player inherits Rectangle {
    background: @linear-gradient(180deg, State.bg-first, State.bg-mid 40%, State.bg-last);
    Flickable { width: parent.width; height: parent.height; content-width: self.width;
        VerticalLayout { width: parent.width; padding: 16px; spacing: 10px;
            HorizontalLayout { height: 44px;
                Action { width: 44px; text: "Recolher player"; icon: Icons.collapse; clicked => { State.expanded = false; } }
                Action { text: "Áudio"; bright: true; }
                Action { text: "Vídeo"; clicked => { State.command("video"); } }
            }
            Rectangle { height: min(290px, self.width); width: min(290px, parent.width - 32px); border-radius: 26px; background: @linear-gradient(135deg, State.songs[State.current].first, State.songs[State.current].last);
                if !State.lyrics : Text { y: parent.height - 74px; height: 60px; width: parent.width; text: State.album; color: #241612; font-size: 26px; font-family: "serif"; font-weight: 700; horizontal-alignment: center; overflow: elide; }
                if !State.lyrics : TouchArea { clicked => { State.lyrics = true; } }
                if State.lyrics : Glass { VerticalLayout { padding: 12px;
                    Action { text: "LETRA AO VIVO · Voltar à capa"; clicked => { State.lyrics = false; } }
                    Flickable { content-width: self.width; VerticalLayout { width: parent.width; spacing: 12px; for line[i] in State.lines : Text { text: line; font-size: i == 2 || i == 3 ? 17px : 15px; color: i == 2 || i == 3 ? Theme.ink : Theme.muted; wrap: word-wrap; } } }
                } }
            }
            HorizontalLayout { height: 56px;
                VerticalLayout { Text { text: State.songs[State.current].title; color: Theme.ink; font-size: 21px; font-weight: 800; overflow: elide; } Text { text: State.songs[State.current].artist; color: Theme.muted; font-size: 14px; } }
                Action { width: 44px; text: "Compartilhar"; icon: Icons.share; clicked => { State.command("share"); } }
                Action { width: 44px; text: State.liked ? "♥" : "♡"; clicked => { State.liked = !State.liked; } }
            }
            Glass { height: 300px; border-radius: 28px;
                VerticalLayout { padding: 16px; spacing: 2px;
                    Scrubber { label: "Progresso"; value: State.progress; height: 44px; changed(v) => { State.seek(v); } }
                    HorizontalLayout { Text { text: State.position; color: Theme.muted; font-size: 12px; } Text { text: State.remaining; color: Theme.muted; font-size: 12px; horizontal-alignment: right; } }
                    HorizontalLayout { Action { text: "Modo aleatório"; icon: Icons.shuffle; active: State.shuffle; clicked => { State.shuffle = !State.shuffle; } } Action { text: "Faixa anterior"; icon: Icons.previous; clicked => { State.command("previous"); } } Action { text: State.playing ? "Pausar" : "Reproduzir"; icon: State.playing ? Icons.pause : Icons.play; width: 60px; height: 60px; border-radius: 30px; bright: true; clicked => { State.command("play"); } } Action { text: "Próxima faixa"; icon: Icons.next; clicked => { State.command("next"); } } Action { text: "Repetir faixa"; icon: Icons.repeat; active: State.repeat; clicked => { State.repeat = !State.repeat; } } }
                    Scrubber { label: "Volume"; thickness: 4px; value: State.volume; height: 44px; changed(v) => { State.volume-changed(v); } }
                    HorizontalLayout { Action { text: "Fila"; clicked => { State.queue = true; } } Action { text: "Letra"; active: State.lyrics; clicked => { State.lyrics = !State.lyrics; } } Action { text: "Timer"; clicked => { State.command("timer"); } } Action { text: "•••"; clicked => { State.command("more"); } } }
                    Action { text: "This phone"; clicked => { State.command("device"); } }
                }
            }
        }
    }
}
```

## ui/app.slint

```slint
import { Icons } from "icons.slint";
import { Theme } from "theme.slint";
import { State, Song, Setting } from "state.slint";
export { Theme, State, Song, Setting }
import { Inicio } from "screens/inicio.slint";
import { Explorar } from "screens/explorar.slint";
import { Biblioteca } from "screens/biblioteca.slint";
import { Pesquisa } from "screens/pesquisa.slint";
import { Configuracoes } from "screens/configuracoes.slint";
import { Player } from "screens/player.slint";
import { Island } from "components/island.slint";
import { Action } from "components/action.slint";
import { Queue } from "components/queue.slint";
import { Toast } from "components/toast.slint";
import { Confirmation } from "components/confirmation.slint";
export component AppWindow inherits Window {
    title: "kazzu · Protótipo simulado"; default-font-size: 14px; preferred-width: 420px; preferred-height: 860px; min-width: 300px; min-height: 480px; background: #0b0a11;
    device := Rectangle {
        width: min(420px, root.width); height: min(900px, root.height); x: (root.width - self.width) / 2; y: (root.height - self.height) / 2;
        background: @linear-gradient(150deg, Theme.accent.with-alpha(0.18), Theme.base 45%, Theme.base); clip: true;
        FocusScope { width: parent.width; height: parent.height;
            init => { self.focus(); }
            key-pressed(e) => {
                if (e.text == Key.Escape || e.text == Key.Back) { if (State.confirm != 0) { State.confirm = 0; } else if (State.queue) { State.queue = false; } else if (State.expanded) { State.expanded = false; } else if (State.page == 4) { State.go(State.origin); } return accept; }
                if (State.confirm != 0) { return reject; }
                if (e.text == " ") { State.command("play"); return accept; }
                if (e.text == "j" || e.text == "J") { State.command("previous"); return accept; }
                if (e.text == "l" || e.text == "L") { State.command("next"); return accept; }
                if (e.text == "s" || e.text == "S") { State.command("shuffle"); return accept; }
                if (e.text == "r" || e.text == "R") { State.command("repeat"); return accept; }
                if (e.text == "m" || e.text == "M") { State.command("mute"); return accept; }
                return reject;
            }
            VerticalLayout { padding: 16px; spacing: 12px;
                HorizontalLayout { height: 48px;
                    Action { font-size: State.page == 1 ? 32px : State.page == 0 ? 22px : 18px; font-weight: 800; text: State.page == 0 ? "kazzu" : State.page == 1 ? "Explorar" : State.page == 2 ? "Biblioteca" : State.page == 3 ? "Pesquisa" : "‹ Configurações"; clicked => { if (State.page == 4) { State.go(State.origin); } } }
                    Action { width: 44px; text: "Histórico"; icon: Icons.history; clicked => { State.command("history"); } }
                    Action { width: 44px; text: "Configurações"; icon: Icons.settings; clicked => { State.go(4); } }
                }
                Flickable { content-width: self.width;
                    VerticalLayout { width: parent.width; padding-bottom: 170px;
                        if State.page == 0 : Inicio { }
                        if State.page == 1 : Explorar { }
                        if State.page == 2 : Biblioteca { }
                        if State.page == 3 : Pesquisa { }
                        if State.page == 4 : Configuracoes { }
                    }
                }
            }
        }
        Island { x: 10px; y: parent.height - self.height - 10px; width: parent.width - 20px; }
        if State.expanded : Player { width: parent.width; height: parent.height; }
        if State.queue : Queue { width: parent.width; height: parent.height * 0.6; y: parent.height - self.height; }
        if State.toast != "" : Toast { x: 20px; y: parent.height - 200px; width: parent.width - 40px; message: State.toast; }
        if State.confirm != 0 : Confirmation { width: parent.width; height: parent.height; }
    }
}
```

## src/models.rs

```rust
use serde::Deserialize;
#[derive(Clone, Deserialize)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: f32,
    #[serde(rename = "pos")]
    pub position: f32,
    #[serde(rename = "artGradient")]
    pub art_gradient: String,
    #[serde(rename = "bgGradient")]
    pub background: String,
    pub lyrics: Vec<String>,
}
#[derive(Default)]
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<usize>,
}
pub struct Settings {
    pub quality: usize,
    pub crossfade: usize,
    pub toggles: [bool; 4],
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            quality: 0,
            crossfade: 2,
            toggles: [true; 4],
        }
    }
}
```

## src/audio.rs

```rust
/// Transporte simulado. Uma implementação real pode substituir este contrato.
pub trait Audio {
    fn playing(&self) -> bool;
    fn set_playing(&mut self, value: bool);
    fn volume(&self) -> f32;
    fn set_volume(&mut self, value: f32);
    fn advance(&self, position: &mut f32, elapsed: f32);
}
pub struct SimulatedAudio {
    playing: bool,
    volume: f32,
}
impl Default for SimulatedAudio {
    fn default() -> Self {
        Self {
            playing: false,
            volume: 0.70,
        }
    }
}
impl Audio for SimulatedAudio {
    fn playing(&self) -> bool {
        self.playing
    }
    fn set_playing(&mut self, value: bool) {
        self.playing = value;
    }
    fn volume(&self) -> f32 {
        self.volume
    }
    fn set_volume(&mut self, value: f32) {
        self.volume = value.clamp(0.0, 1.0);
    }
    fn advance(&self, position: &mut f32, elapsed: f32) {
        if self.playing {
            *position += elapsed;
        }
    }
}
```

## src/main.rs

```rust
mod audio;
mod models;
mod snapshot;
use audio::{Audio, SimulatedAudio};
use models::{Settings, Track};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
slint::include_modules!();
struct Controller {
    tracks: Vec<Track>,
    index: usize,
    audio: SimulatedAudio,
    settings: Settings,
    rows: Vec<serde_json::Value>,
    visible_rows: Vec<usize>,
    palettes: Vec<serde_json::Value>,
    seed: u64,
    toast_until: Instant,
    scan_until: Option<Instant>,
    scan_reset: Option<Instant>,
    clipboard: Option<arboard::Clipboard>,
}
fn color(value: &str) -> slint::Color {
    let value = u32::from_str_radix(value.trim_start_matches('#'), 16).unwrap_or(0);
    slint::Color::from_rgb_u8((value >> 16) as u8, (value >> 8) as u8, value as u8)
}
fn rgba(css: &str) -> slint::Color {
    let parts: Vec<f32> = css
        .trim_start_matches("rgba(")
        .trim_end_matches(')')
        .split(',')
        .filter_map(|part| part.trim().parse().ok())
        .collect();
    slint::Color::from_argb_u8(
        (parts[3] * 255.0) as u8,
        parts[0] as u8,
        parts[1] as u8,
        parts[2] as u8,
    )
}
fn colors(gradient: &str) -> Vec<slint::Color> {
    gradient
        .split('#')
        .skip(1)
        .map(|s| color(&s[..6]))
        .collect()
}
fn time(value: f32) -> String {
    let value = value.max(0.0) as u32;
    format!("{}:{:02}", value / 60, value % 60)
}
fn songs(tracks: &[Track]) -> ModelRc<Song> {
    ModelRc::new(VecModel::from(
        tracks
            .iter()
            .map(|t| {
                let c = colors(&t.art_gradient);
                Song {
                    title: t.title.as_str().into(),
                    artist: t.artist.as_str().into(),
                    duration: time(t.duration).into(),
                    first: c[0],
                    last: *c.last().unwrap(),
                }
            })
            .collect::<Vec<_>>(),
    ))
}
impl Controller {
    fn search(&mut self, ui: &AppWindow, query: &str) {
        let query = query.to_lowercase();
        let results = self
            .tracks
            .iter()
            .filter(|t| {
                !query.is_empty()
                    && format!("{} {} {}", t.title, t.artist, t.lyrics.join(" "))
                        .to_lowercase()
                        .contains(&query)
            })
            .cloned()
            .collect::<Vec<_>>();
        ui.global::<State>().set_results(songs(&results));
    }
    fn publish(&self, ui: &AppWindow) {
        let s = ui.global::<State>();
        let t = &self.tracks[self.index];
        s.set_current(self.index as i32);
        s.set_playing(self.audio.playing());
        s.set_volume(self.audio.volume());
        s.set_progress(t.position / t.duration);
        s.set_position(time(t.position).into());
        s.set_remaining(format!("-{}", time(t.duration - t.position)).into());
        s.set_album(t.album.as_str().into());
        s.set_lines(ModelRc::new(VecModel::from(
            t.lyrics
                .iter()
                .map(|s| s.as_str().into())
                .collect::<Vec<slint::SharedString>>(),
        )));
        let atmosphere = colors(&t.background);
        s.set_bg_first(atmosphere[0]);
        s.set_bg_mid(atmosphere[1]);
        s.set_bg_last(atmosphere[2]);
    }
    fn toast(&mut self, ui: &AppWindow, message: &str) {
        ui.global::<State>().set_toast(message.into());
        self.toast_until = Instant::now() + Duration::from_millis(2200);
    }
    fn random(&mut self, exclude_current: bool) -> usize {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        if exclude_current {
            (self.index + 1 + self.seed as usize % (self.tracks.len() - 1)) % self.tracks.len()
        } else {
            self.seed as usize % self.tracks.len()
        }
    }
    fn next(&mut self, ui: &AppWindow) {
        self.index = if ui.global::<State>().get_shuffle() {
            self.random(true)
        } else {
            (self.index + 1) % self.tracks.len()
        };
        self.tracks[self.index].position = 0.0;
        self.audio.set_playing(true);
    }
    fn rows(&mut self, ui: &AppWindow) {
        let query = ui.global::<State>().get_settings_query().to_lowercase();
        self.visible_rows = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                format!("{} {}", r["title"], r["sub"])
                    .to_lowercase()
                    .contains(&query)
            })
            .map(|(i, _)| i)
            .collect();
        let rows = self
            .visible_rows
            .iter()
            .map(|&i| {
                let r = &self.rows[i];
                let action = r["action"].as_str().unwrap_or("");
                let toggle_index = [
                    "toggleNormalization",
                    "toggleLiquidGlass",
                    "toggleAnimations",
                    "toggleHistory",
                ]
                .iter()
                .position(|key| action.contains(key));
                Setting {
                    title: r["title"].as_str().unwrap_or("").into(),
                    subtitle: r["sub"].as_str().unwrap_or("").into(),
                    value: r["value"].as_str().unwrap_or("").into(),
                    group: r["group"].as_str().unwrap_or("").into(),
                    toggle: toggle_index.is_some(),
                    checked: toggle_index.is_some_and(|i| self.settings.toggles[i]),
                }
            })
            .collect::<Vec<_>>();
        ui.global::<State>()
            .set_settings(ModelRc::new(VecModel::from(rows)));
    }
    fn palette(&mut self, ui: &AppWindow, index: usize) {
        if let Some(p) = self.palettes.get(index) {
            let theme = ui.global::<Theme>();
            theme.set_accent(color(p["primary"].as_str().unwrap()));
            theme.set_secondary(color(p["secondary"].as_str().unwrap()));
            theme.set_base(color(p["baseDark"].as_str().unwrap()));
            theme.set_card(rgba(p["cardBg"].as_str().unwrap()));
            ui.global::<State>().set_palette(index as i32);
            self.rows[4]["sub"] = p["sub"].clone();
            self.rows[4]["value"] = p["name"].clone();
        }
        self.rows(ui);
    }
    fn setting(&mut self, ui: &AppWindow, visible_index: usize) {
        let Some(&i) = self.visible_rows.get(visible_index) else {
            return;
        };
        let action = self.rows[i]["action"].as_str().unwrap_or("").to_owned();
        if action.contains("cycleAudioQuality") {
            self.settings.quality = (self.settings.quality + 1) % 4;
            let (label, pill) = [
                ("Alta fidelidade Lossless • 320 kbps", "320k"),
                ("Master Hi-Res FLAC • 24-bit/96kHz", "Hi-Res"),
                ("Padrão Econômico • 160 kbps", "160k"),
                ("Automático (Adapta à conexão)", "Auto"),
            ][self.settings.quality];
            self.rows[i]["sub"] = label.into();
            self.rows[i]["value"] = pill.into();
        } else if action.contains("cycleCrossfade") {
            self.settings.crossfade = (self.settings.crossfade + 1) % 5;
            let seconds = self.settings.crossfade * 2;
            self.rows[i]["value"] = format!("{seconds}s").into();
            self.rows[i]["sub"] = if seconds == 0 {
                "Desativado (Corte seco)".into()
            } else {
                format!("Transição suave de {seconds}s entre músicas").into()
            };
        } else if action.contains("cycleThemeNext") {
            self.palette(ui, (ui.global::<State>().get_palette() as usize + 1) % 6);
        } else if action.contains("toggleSetting") {
            let index = [
                "toggleNormalization",
                "toggleLiquidGlass",
                "toggleAnimations",
                "toggleHistory",
            ]
            .iter()
            .position(|k| action.contains(k))
            .unwrap();
            self.settings.toggles[index] = !self.settings.toggles[index];
            ui.global::<Theme>().set_glass(self.settings.toggles[1]);
        } else if action.contains("scanLibraryNow") {
            self.rows[i]["sub"] = "Indexando arquivos locais de áudio...".into();
            self.rows[i]["value"] = "Buscando...".into();
            self.scan_until = Some(Instant::now() + Duration::from_millis(1200));
        } else if action.contains("clearMusicCache") {
            ui.global::<State>().set_confirm(1);
        } else if action.contains("clearSearchHistory") {
            ui.global::<State>().set_confirm(2);
        } else if let Some(message) = action
            .strip_prefix("showToast('")
            .and_then(|s| s.strip_suffix("')"))
        {
            self.toast(ui, &message.replace("\\n", "\n"));
        }
        self.rows(ui);
    }
    fn command(&mut self, ui: &AppWindow, command: &str) {
        match command {
            "play" => self.audio.set_playing(!self.audio.playing()),
            "shuffle" => ui
                .global::<State>()
                .set_shuffle(!ui.global::<State>().get_shuffle()),
            "repeat" => ui
                .global::<State>()
                .set_repeat(!ui.global::<State>().get_repeat()),
            "mute" => self
                .audio
                .set_volume(if self.audio.volume() > 0.0 { 0.0 } else { 0.7 }),
            "next" => self.next(ui),
            "previous" => {
                if self.tracks[self.index].position <= 4.0 {
                    self.index = (self.index + self.tracks.len() - 1) % self.tracks.len();
                    self.audio.set_playing(true);
                }
                self.tracks[self.index].position = 0.0;
            }
            "random" => {
                self.index = self.random(false);
                self.tracks[self.index].position = 0.0;
                self.audio.set_playing(true);
            }
            "clear-cache" => {
                self.rows[9]["sub"] = "0 KB utilizados • 0 faixas em cache".into();
                self.rows(ui);
                self.toast(ui, "Cache limpo com sucesso");
            }
            "clear-history" => {
                ui.global::<State>().set_history(false);
                self.toast(ui, "Histórico de buscas apagado");
            }
            "share" => {
                let track = &self.tracks[self.index];
                let text = format!("Ouvindo {} de {} no kazzu!", track.title, track.artist);
                if self.clipboard.is_none() {
                    self.clipboard = arboard::Clipboard::new().ok();
                }
                match self
                    .clipboard
                    .as_mut()
                    .ok_or(())
                    .and_then(|clipboard| clipboard.set_text(text).map_err(|_| ()))
                {
                    Ok(()) => self.toast(ui, "Compartilhamento copiado!"),
                    Err(_) => self.toast(ui, "Não foi possível acessar a área de transferência"),
                }
            }
            "video" => self.toast(ui, "Reprodução em vídeo indisponível nesta versão"),
            "timer" => self.toast(ui, "Sleep Timer: Desligar após 30 minutos"),
            "more" => self.toast(
                ui,
                "Opções: Adicionar à Playlist, Ver Álbum, Ajustes de Áudio, Equalizador",
            ),
            "device" => self.toast(ui, "Saída: Alto-falantes internos (This phone)"),
            "profile" => self.toast(ui, "Perfil: cuscusdoce rael\nMembro Free desde Out/2024"),
            "history" => self.toast(ui, "Histórico de reprodução"),
            "playlist" => self.toast(ui, "Criação de playlists indisponível neste protótipo"),
            command if command.starts_with("find:") => {
                if let Some(index) = self.tracks.iter().position(|t| t.title == &command[5..]) {
                    self.index = index;
                    self.tracks[index].position = 0.0;
                    self.audio.set_playing(true);
                }
            }
            message => self.toast(ui, &format!("Abrindo {message}")),
        }
        self.publish(ui);
    }
}
fn main() -> Result<(), slint::PlatformError> {
    let snapshot_dir = std::env::args().skip(1).find_map(|arg| {
        arg.strip_prefix("--snapshot-dir=")
            .map(std::path::PathBuf::from)
    });
    let snapshot_window = snapshot_dir.as_ref().map(|_| snapshot::install());
    let ui = AppWindow::new()?;
    let controller = Rc::new(RefCell::new(Controller {
        tracks: serde_json::from_str(include_str!("tracks.json")).expect("catálogo do protótipo"),
        index: 0,
        audio: SimulatedAudio::default(),
        settings: Settings::default(),
        rows: serde_json::from_str(include_str!("settings.json")).expect("ajustes"),
        visible_rows: vec![],
        palettes: serde_json::from_str(include_str!("palettes.json")).expect("paletas"),
        seed: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
            | 1,
        toast_until: Instant::now(),
        scan_until: None,
        scan_reset: None,
        clipboard: None,
    }));
    ui.global::<State>()
        .set_songs(songs(&controller.borrow().tracks));
    controller.borrow().publish(&ui);
    controller.borrow_mut().rows(&ui);
    // A playlist is a model ready for future creation; the HTML provides no creation flow.
    let playlist = models::Playlist {
        name: "💿 Continue ouvindo".into(),
        tracks: vec![0, 1, 3],
    };
    ui.global::<State>()
        .set_continue_title(playlist.name.into());
    ui.global::<State>()
        .set_continue_tracks(ModelRc::new(VecModel::from(
            playlist
                .tracks
                .into_iter()
                .map(|index| index as i32)
                .collect::<Vec<_>>(),
        )));
    macro_rules! bind { ($method:ident, |$c:ident, $w:ident $(, $arg:ident)*| $body:block) => {{
        let weak = ui.as_weak(); let state = controller.clone();
        ui.global::<State>().$method(move |$($arg),*| { if let Some($w) = weak.upgrade() { let mut $c = state.borrow_mut(); $body } });
    }}; }
    bind!(on_command, |c, w, command| {
        c.command(&w, &command);
    });
    bind!(on_select, |c, w, index, preserve| {
        if (0..c.tracks.len() as i32).contains(&index) {
            c.index = index as usize;
            if !preserve {
                c.tracks[index as usize].position = 0.0;
            }
            c.audio.set_playing(true);
            c.publish(&w);
        }
    });
    bind!(on_seek, |c, w, value| {
        let index = c.index;
        c.tracks[index].position = value.clamp(0.0, 1.0) * c.tracks[index].duration;
        c.publish(&w);
    });
    bind!(on_volume_changed, |c, w, value| {
        c.audio.set_volume(value);
        c.publish(&w);
    });
    bind!(on_theme, |c, w, index| {
        c.palette(&w, index as usize);
    });
    bind!(on_setting, |c, w, index| {
        c.setting(&w, index as usize);
    });
    bind!(on_filter_settings, |c, w, _query| {
        c.rows(&w);
    });
    bind!(on_search, |c, w, query| {
        c.search(&w, &query);
    });
    let timer = slint::Timer::default();
    let weak = ui.as_weak();
    let state = controller.clone();
    let last = Rc::new(RefCell::new(Instant::now()));
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(100),
        move || {
            if let Some(w) = weak.upgrade() {
                let now = Instant::now();
                let elapsed = now.duration_since(*last.borrow()).as_secs_f32();
                *last.borrow_mut() = now;
                let mut c = state.borrow_mut();
                let index = c.index;
                let mut position = c.tracks[index].position;
                c.audio.advance(&mut position, elapsed);
                c.tracks[index].position = position;
                if position >= c.tracks[index].duration {
                    if w.global::<State>().get_repeat() {
                        c.tracks[index].position = 0.0;
                    } else {
                        c.next(&w);
                    }
                }
                if now >= c.toast_until {
                    w.global::<State>().set_toast("".into());
                }
                if c.scan_until.is_some_and(|deadline| now >= deadline) {
                    c.scan_until = None;
                    c.scan_reset = Some(now + Duration::from_millis(2500));
                    c.rows[8]["sub"] = "742 músicas locais encontradas • Atualizado agora".into();
                    c.rows[8]["value"] = "Concluído!".into();
                    c.rows(&w);
                }
                if c.scan_reset.is_some_and(|deadline| now >= deadline) {
                    c.scan_reset = None;
                    c.rows[8]["value"] = "Escanear".into();
                    c.rows(&w);
                }
                c.publish(&w);
            }
        },
    );
    if let (Some(directory), Some(window)) = (snapshot_dir, snapshot_window) {
        snapshot::capture(&ui, &window, &directory).expect("gravar capturas Slint");
        return Ok(());
    }
    ui.run()
}
```

## ui/state.slint

```slint
export struct Song { title: string, artist: string, duration: string, first: color, last: color }
export struct Setting { title: string, subtitle: string, value: string, toggle: bool, checked: bool, group: string }
export global State {
    in-out property <int> page: 1;
    in-out property <int> origin: 1;
    in-out property <bool> expanded;
    in-out property <bool> queue;
    in-out property <bool> lyrics;
    in-out property <bool> playing;
    in-out property <bool> liked: true;
    in-out property <bool> shuffle;
    in-out property <bool> repeat;
    in-out property <int> current;
    in-out property <float> progress;
    in-out property <float> volume: 0.7;
    in-out property <string> position;
    in-out property <string> remaining;
    in-out property <string> album;
    in-out property <color> bg-first;
    in-out property <color> bg-mid;
    in-out property <color> bg-last;
    in-out property <[Song]> songs;
    in-out property <[Song]> results;
    in-out property <[int]> continue-tracks;
    in-out property <string> continue-title;
    in-out property <[string]> lines;
    in-out property <[Setting]> settings;
    in-out property <string> query;
    in-out property <string> settings-query;
    in-out property <string> toast;
    in-out property <int> confirm: 0;
    in-out property <bool> history: true;
    in-out property <int> palette;
    in-out property <int> chip;
    callback command(string);
    callback select(int, bool);
    callback seek(float);
    callback volume-changed(float);
    callback setting(int);
    callback theme(int);
    callback search(string);
    callback filter-settings(string);
    public function go(index: int) { if (index != 4) { origin = index; } page = index; }
}
```

## ui/icons.slint

```slint
export global Icons {
    out property <image> previous: @image-url("icons/previous.svg");
    out property <image> next: @image-url("icons/next.svg");
    out property <image> shuffle: @image-url("icons/shuffle.svg");
    out property <image> repeat: @image-url("icons/repeat.svg");
    out property <image> share: @image-url("icons/share.svg");
    out property <image> heart: @image-url("icons/heart.svg");
    out property <image> collapse: @image-url("icons/collapse.svg");
    out property <image> queue: @image-url("icons/queue.svg");
    out property <image> lyrics: @image-url("icons/lyrics.svg");
    out property <image> timer: @image-url("icons/timer.svg");
    out property <image> more: @image-url("icons/more.svg");
    out property <image> search: @image-url("icons/search.svg");
    out property <image> settings: @image-url("icons/settings.svg");
    out property <image> history: @image-url("icons/history.svg");
    out property <image> play: @image-url("icons/play.svg");
    out property <image> pause: @image-url("icons/pause.svg");
}
```

## src/snapshot.rs

```rust
use crate::{AppWindow, State};
use slint::platform::software_renderer::{
    MinimalSoftwareWindow, PremultipliedRgbaColor, RepaintBufferType,
};
use slint::{ComponentHandle, PhysicalSize};
use std::{io::Write, path::Path, rc::Rc};
struct Platform(Rc<MinimalSoftwareWindow>);
impl slint::platform::Platform for Platform {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
}
pub fn install() -> Rc<MinimalSoftwareWindow> {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Platform(window.clone()))).expect("renderer de captura");
    window
}
fn verify_callbacks(ui: &AppWindow) {
    let state = ui.global::<State>();
    state.invoke_select(0, false);
    assert_eq!(state.get_current(), 0);
    assert!(state.get_playing());
    state.invoke_seek(0.5);
    assert!((state.get_progress() - 0.5).abs() < 0.001);
    state.invoke_command("previous".into());
    assert_eq!(state.get_current(), 0);
    assert_eq!(state.get_progress(), 0.0);
    state.invoke_command("previous".into());
    assert_eq!(state.get_current(), 7);
    state.set_shuffle(true);
    for _ in 0..64 {
        let previous = state.get_current();
        state.invoke_command("next".into());
        assert_ne!(state.get_current(), previous);
    }
    state.invoke_volume_changed(1.5);
    assert_eq!(state.get_volume(), 1.0);
    state.invoke_volume_changed(-1.0);
    assert_eq!(state.get_volume(), 0.0);
    let solar = ui.global::<crate::Theme>().get_accent();
    state.invoke_theme(1);
    assert_ne!(ui.global::<crate::Theme>().get_accent(), solar);
    state.invoke_theme(0);
    state.invoke_select(0, false);
    state.invoke_command("play".into());
    state.set_shuffle(false);
    state.invoke_volume_changed(0.7);
    println!("Callbacks verificados: transporte, anterior, seek, shuffle, volume e tema.");
}
pub fn capture(
    ui: &AppWindow,
    window: &MinimalSoftwareWindow,
    directory: &Path,
) -> std::io::Result<()> {
    verify_callbacks(ui);
    std::fs::create_dir_all(directory)?;
    ui.show().map_err(std::io::Error::other)?;
    for width in [320, 420, 900] {
        window.set_size(PhysicalSize::new(width, 860));
        for (name, page, player, queue) in [
            ("inicio", 0, false, false),
            ("explorar", 1, false, false),
            ("biblioteca", 2, false, false),
            ("pesquisa", 3, false, false),
            ("configuracoes", 4, false, false),
            ("player", 0, true, false),
            ("fila", 0, true, true),
        ] {
            let s = ui.global::<State>();
            s.set_page(page);
            s.set_expanded(player);
            s.set_queue(queue);
            slint::platform::update_timers_and_animations();
            let mut pixels = vec![PremultipliedRgbaColor::default(); width as usize * 860];
            window.draw_if_needed(|renderer| {
                renderer.render(&mut pixels, width as usize);
            });
            let mut file = std::fs::File::create(directory.join(format!("{width}-{name}.ppm")))?;
            write!(file, "P6\n{width} 860\n255\n")?;
            for pixel in pixels {
                file.write_all(&[pixel.red, pixel.green, pixel.blue])?;
            }
        }
    }
    Ok(())
}
```

## src/palettes.json

```json
[
  {
    "name": "Laranja Solar",
    "tag": "Solar Amber",
    "sub": "Escuro (Liquid Glass Laranja Solar)",
    "primary": "#ff6b00",
    "secondary": "#ffa858",
    "glow": "rgba(255, 107, 0, 0.45)",
    "borderGlow": "rgba(255, 140, 0, 0.28)",
    "bgTint1": "rgba(255, 107, 0, 0.18)",
    "bgTint2": "rgba(255, 140, 0, 0.12)",
    "baseDark": "#120a05",
    "cardBg": "rgba(30, 18, 10, 0.72)",
    "pillBg": "rgba(255, 140, 0, 0.12)",
    "chipActiveBg": "#fdf6ec",
    "chipActiveText": "#12101b"
  },
  {
    "name": "Rosa Cyber / Neon Berry",
    "tag": "Neon Berry",
    "sub": "Escuro (Rosa Cyber & Neon Berry)",
    "primary": "#ff3377",
    "secondary": "#ff7fa8",
    "glow": "rgba(255, 51, 119, 0.45)",
    "borderGlow": "rgba(255, 51, 119, 0.32)",
    "bgTint1": "rgba(255, 51, 119, 0.20)",
    "bgTint2": "rgba(232, 62, 111, 0.14)",
    "baseDark": "#160710",
    "cardBg": "rgba(36, 14, 25, 0.72)",
    "pillBg": "rgba(255, 51, 119, 0.14)",
    "chipActiveBg": "#ff3377",
    "chipActiveText": "#ffffff"
  },
  {
    "name": "Ametista Profundo",
    "tag": "Roxo Elétrico",
    "sub": "Escuro (Ametista Profundo & Lavanda)",
    "primary": "#9d4edd",
    "secondary": "#c77dff",
    "glow": "rgba(157, 78, 221, 0.45)",
    "borderGlow": "rgba(199, 125, 255, 0.30)",
    "bgTint1": "rgba(157, 78, 221, 0.20)",
    "bgTint2": "rgba(123, 44, 191, 0.14)",
    "baseDark": "#11071c",
    "cardBg": "rgba(28, 14, 42, 0.74)",
    "pillBg": "rgba(157, 78, 221, 0.15)",
    "chipActiveBg": "#9d4edd",
    "chipActiveText": "#ffffff"
  },
  {
    "name": "Esmeralda Liquid",
    "tag": "Menta Liquid",
    "sub": "Escuro (Esmeralda Liquid & Menta Néon)",
    "primary": "#10b981",
    "secondary": "#00f5d4",
    "glow": "rgba(0, 245, 212, 0.40)",
    "borderGlow": "rgba(16, 185, 129, 0.32)",
    "bgTint1": "rgba(0, 245, 212, 0.18)",
    "bgTint2": "rgba(16, 185, 129, 0.12)",
    "baseDark": "#041410",
    "cardBg": "rgba(10, 32, 26, 0.74)",
    "pillBg": "rgba(0, 245, 212, 0.14)",
    "chipActiveBg": "#10b981",
    "chipActiveText": "#ffffff"
  },
  {
    "name": "Azul Cobalto",
    "tag": "Ciano Deep",
    "sub": "Escuro (Azul Cobalto & Ciano Elétrico)",
    "primary": "#3a86ff",
    "secondary": "#00b4d8",
    "glow": "rgba(58, 134, 255, 0.45)",
    "borderGlow": "rgba(0, 180, 216, 0.32)",
    "bgTint1": "rgba(58, 134, 255, 0.20)",
    "bgTint2": "rgba(0, 180, 216, 0.14)",
    "baseDark": "#070e1c",
    "cardBg": "rgba(12, 24, 46, 0.75)",
    "pillBg": "rgba(58, 134, 255, 0.15)",
    "chipActiveBg": "#3a86ff",
    "chipActiveText": "#ffffff"
  },
  {
    "name": "Obsidian & Ouro",
    "tag": "Champagne Lux",
    "sub": "Escuro Minimal (Obsidian & Dourado Champagne)",
    "primary": "#e0a96d",
    "secondary": "#ffd166",
    "glow": "rgba(255, 209, 102, 0.40)",
    "borderGlow": "rgba(224, 169, 109, 0.32)",
    "bgTint1": "rgba(255, 209, 102, 0.15)",
    "bgTint2": "rgba(224, 169, 109, 0.12)",
    "baseDark": "#0a0a0c",
    "cardBg": "rgba(26, 24, 22, 0.78)",
    "pillBg": "rgba(255, 209, 102, 0.14)",
    "chipActiveBg": "#ffd166",
    "chipActiveText": "#12101b"
  }
]
```

## src/settings.json

```json
[
  {
    "title": "Qualidade de áudio",
    "sub": "Alta fidelidade Lossless • 320 kbps",
    "action": "cycleAudioQuality()",
    "value": "320k",
    "toggle": false,
    "group": "🎵 Reprodução & Áudio"
  },
  {
    "title": "Normalização de volume",
    "sub": "Mesmo nível sonoro entre faixas",
    "action": "toggleSetting('toggleNormalization')",
    "value": "",
    "toggle": true,
    "group": "🎵 Reprodução & Áudio"
  },
  {
    "title": "Transição contínua (Crossfade)",
    "sub": "Fusão suave entre músicas",
    "action": "cycleCrossfade()",
    "value": "4s",
    "toggle": false,
    "group": "🎵 Reprodução & Áudio"
  },
  {
    "title": "Saída de áudio padrão",
    "sub": "Alto-falante integrado (Este celular)",
    "action": "showToast('Seletor de saída: Este celular (Padrão) / Bluetooth / Cast')",
    "value": "",
    "toggle": false,
    "group": "🎵 Reprodução & Áudio"
  },
  {
    "title": "Tema do app",
    "sub": "Escuro (Liquid Glass Laranja Solar)",
    "action": "cycleThemeNext()",
    "value": "Laranja Solar",
    "toggle": false,
    "group": "✨ Aparência & Estilo"
  },
  {
    "title": "Efeito Liquid Glass",
    "sub": "Desfoque dinâmico e refrações em tempo real",
    "action": "toggleSetting('toggleLiquidGlass')",
    "value": "",
    "toggle": true,
    "group": "✨ Aparência & Estilo"
  },
  {
    "title": "Animações fluidas",
    "sub": "Transições suaves de 60fps",
    "action": "toggleSetting('toggleAnimations')",
    "value": "",
    "toggle": true,
    "group": "✨ Aparência & Estilo"
  },
  {
    "title": "Pastas locais indexadas",
    "sub": "/storage/emulated/0/Music",
    "action": "showToast('Pastas indexadas:\\n/storage/emulated/0/Music\\n/storage/emulated/0/Download')",
    "value": "",
    "toggle": false,
    "group": "📁 Fontes de Música & Biblioteca"
  },
  {
    "title": "Escanear biblioteca agora",
    "sub": "Último escaneamento hoje às 14:20",
    "action": "scanLibraryNow()",
    "value": "Escanear",
    "toggle": false,
    "group": "📁 Fontes de Música & Biblioteca"
  },
  {
    "title": "Cache offline e downloads",
    "sub": "2.4 GB utilizados • 82 faixas",
    "action": "clearMusicCache()",
    "value": "Limpar",
    "toggle": false,
    "group": "📁 Fontes de Música & Biblioteca"
  },
  {
    "title": "Histórico de reprodução e buscas",
    "sub": "Personalizar recomendações e estatísticas",
    "action": "toggleSetting('toggleHistory')",
    "value": "",
    "toggle": true,
    "group": "🔒 Privacidade & Dados"
  },
  {
    "title": "Limpar histórico recente",
    "sub": "Apaga buscas recentes do dispositivo",
    "action": "clearSearchHistory()",
    "value": "",
    "toggle": false,
    "group": "🔒 Privacidade & Dados"
  },
  {
    "title": "Atalhos de teclado",
    "sub": "Comandos rápidos para teclado físico",
    "action": "showToast('Atalhos de Teclado kazzu:\\nEspaço: Play / Pause\\nJ / L: Voltar / Avançar\\nS: Modo Aleatório\\nR: Repetir\\nM: Mudo')",
    "value": "",
    "toggle": false,
    "group": "ℹ️ Sobre & Atalhos"
  },
  {
    "title": "Versão do kazzu",
    "sub": "v1.4.2 (Liquid Glass Edition) • Código Aberto",
    "action": "showToast('kazzu Music Player\\nVersão 1.4.2 (Liquid Glass Edition)\\nDesenvolvido com carinho • Licença MIT')",
    "value": "v1.4.2",
    "toggle": false,
    "group": "ℹ️ Sobre & Atalhos"
  }
]
```

## src/tracks.json

```json
[
  {
    "title": "Creep",
    "artist": "Radiohead",
    "album": "Pablo Honey",
    "sub": "PABLO HONEY",
    "duration": 238,
    "pos": 91,
    "bgGradient": "linear-gradient(180deg, #5c431b 0%, #362512 40%, #151119 100%)",
    "artGradient": "radial-gradient(circle at 50% 40%, #f7e6a1 0%, #d4a339 60%, #573f15 100%)",
    "lyrics": [
      "When you were here before",
      "Couldn't look you in the eye",
      "You're just like an angel",
      "Your skin makes me cry",
      "You float like a feather",
      "In a beautiful world",
      "I wish I was special",
      "You're so fucking special",
      "But I'm a creep, I'm a weirdo",
      "What the hell am I doing here?",
      "I don't belong here"
    ]
  },
  {
    "title": "About A Girl",
    "artist": "Nirvana",
    "album": "Bleach",
    "sub": "BLEACH",
    "duration": 168,
    "pos": 76,
    "bgGradient": "linear-gradient(180deg, #6e414c 0%, #462c33 45%, #14111d 100%)",
    "artGradient": "radial-gradient(circle at 50% 30%, #fff7ce 0%, #ecd98a 50%, #b8a95e 100%)",
    "lyrics": [
      "I'll take advantage while",
      "You're hanging out to dry",
      "I need an easy friend",
      "I do, with an ear to lend",
      "I do think you fit this shoe",
      "I do, won't you have a clue"
    ]
  },
  {
    "title": "He Feels Bad",
    "artist": "Helmet",
    "album": "Meantime",
    "sub": "MEANTIME",
    "duration": 244,
    "pos": 112,
    "bgGradient": "linear-gradient(180deg, #6b1418 0%, #3d0a0d 45%, #12090b 100%)",
    "artGradient": "linear-gradient(160deg, #d62828 0%, #7c111c 50%, #200406 100%)",
    "lyrics": [
      "He feels bad, but it won't last long",
      "Turn it over, keep moving strong",
      "Concrete rhythm, distortion wall",
      "Listen to the echo in the hall"
    ]
  },
  {
    "title": "Dumb",
    "artist": "Nirvana",
    "album": "In Utero",
    "sub": "IN UTERO",
    "duration": 173,
    "pos": 24,
    "bgGradient": "linear-gradient(180deg, #544c2c 0%, #332d18 45%, #131211 100%)",
    "artGradient": "radial-gradient(circle at 50% 40%, #e6d8a7 0%, #a89f68 60%, #585334 100%)",
    "lyrics": [
      "I'm not like them, but I can pretend",
      "The sun is gone, but I have a light",
      "The day is done, but I'm having fun",
      "I think I'm dumb or maybe just happy"
    ]
  },
  {
    "title": "Sunshine Indie",
    "artist": "kazzu Curadoria",
    "album": "Clima alegre",
    "sub": "SUNSHINE INDIE",
    "duration": 215,
    "pos": 40,
    "bgGradient": "linear-gradient(180deg, #1b4f2c 0%, #13331d 45%, #0f1c13 100%)",
    "artGradient": "radial-gradient(circle at center, #6fd384 10%, #2f733d 90%)",
    "lyrics": [
      "Sunlight streaming through my window",
      "Acoustic strings and coffee glow",
      "Nothing on my mind today",
      "Just watching sweet clouds drift away"
    ]
  },
  {
    "title": "Feel Good Inc.",
    "artist": "Gorillaz",
    "album": "Demon Days",
    "sub": "DEMON DAYS",
    "duration": 223,
    "pos": 45,
    "bgGradient": "linear-gradient(180deg, #3d4a36 0%, #252e20 45%, #111511 100%)",
    "artGradient": "radial-gradient(circle at 50% 40%, #b3c5a6 0%, #7a8f6d 60%, #394432 100%)",
    "lyrics": [
      "City's breaking down on a camel's back",
      "They just have to go 'cause they don't know wack",
      "So while you fill the streets, it's appealing to see",
      "You won't get out the county, 'cause you're bad and free"
    ]
  },
  {
    "title": "Schism",
    "artist": "TOOL",
    "album": "Lateralus",
    "sub": "LATERALUS",
    "duration": 407,
    "pos": 90,
    "bgGradient": "linear-gradient(180deg, #5c3f2b 0%, #382417 45%, #151114 100%)",
    "artGradient": "radial-gradient(circle at 50% 30%, #ecd3b6 0%, #9e7552 60%, #402613 100%)",
    "lyrics": [
      "I know the pieces fit 'cause I watched them fall away",
      "Mildewed and smoldering, fundamental differing",
      "Pure intention juxtaposed will set two lovers' souls in motion"
    ]
  },
  {
    "title": "Bad Apple!!",
    "artist": "nomico",
    "album": "Alstroemeria",
    "sub": "ALSTROEMERIA",
    "duration": 254,
    "pos": 60,
    "bgGradient": "linear-gradient(180deg, #44375e 0%, #291e3e 45%, #13101b 100%)",
    "artGradient": "radial-gradient(circle at 50% 30%, #dedede 0%, #888888 60%, #222222 100%)",
    "lyrics": [
      "Nagareteku toki no naka de demo",
      "Kedarusa ga hora guruguru mawatte",
      "Watashi kara hanareru kokoro mo",
      "Mienaiwa sou shiranai"
    ]
  }
]
```

## ui/icons/collapse.svg

```xml
<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M7.41 8.59L12 13.17l4.59-4.58L18 10l-6 6-6-6z" /></svg>
```

## ui/icons/heart.svg

```xml
<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z" /></svg>
```

## ui/icons/history.svg

```xml
<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M12 8v4l3 3m6-3a9 9 0 1 1-18 0 9 9 0 0 1 18 0z" /></svg>
```

## ui/icons/lyrics.svg

```xml
<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M20 2H4c-1.1 0-1.99.9-1.99 2L2 22l4-4h14c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm-2 12H6v-2h12v2zm0-3H6V9h12v2zm0-3H6V6h12v2z" /></svg>
```

## ui/icons/more.svg

```xml
<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><circle cx="12" cy="12" r="2" /><circle cx="6" cy="12" r="2" /><circle cx="18" cy="12" r="2" /></svg>
```

## ui/icons/next.svg

```xml
<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z" /></svg>
```

## ui/icons/pause.svg

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/></svg>
```

## ui/icons/play.svg

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M8 5v14l11-7z"/></svg>
```

## ui/icons/previous.svg

```xml
<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M6 6h2v12H6zm3.5 6l8.5 6V6z" /></svg>
```

## ui/icons/queue.svg

```xml
<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M15 6H3v2h12V6zm0 4H3v2h12v-2zM3 16h8v-2H3v2zM17 6v8.18c-.31-.11-.65-.18-1-.18-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3V8h3V6h-5z" /></svg>
```

## ui/icons/repeat.svg

```xml
<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M7 7h10v3l4-4-4-4v3H5v6h2V7zm10 10H7v-3l-4 4 4 4v-3h12v-6h-2v4z" /></svg>
```

## ui/icons/search.svg

```xml
<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><circle cx="11" cy="11" r="7" /><path d="M20 20l-4-4" /></svg>
```

## ui/icons/settings.svg

```xml
<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" /></svg>
```

## ui/icons/share.svg

```xml
<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M18 16.08c-.76 0-1.44.3-1.96.77L8.91 12.7c.05-.23.09-.46.09-.7s-.04-.47-.09-.7l7.05-4.11c.54.5 1.25.81 2.04.81 1.66 0 3-1.34 3-3s-1.34-3-3-3-3 1.34-3 3c0 .24.04.47.09.7L8.04 9.81C7.5 9.31 6.79 9 6 9c-1.66 0-3 1.34-3 3s1.34 3 3 3c.79 0 1.5-.31 2.04-.81l7.12 4.16c-.05.21-.08.43-.08.65 0 1.61 1.31 2.92 2.92 2.92s2.92-1.31 2.92-2.92c0-1.61-1.31-2.92-2.92-2.92z" /></svg>
```

## ui/icons/shuffle.svg

```xml
<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M10.59 9.17L5.41 4 4 5.41l5.17 5.17 1.42-1.41zM14.5 4l2.04 2.04L4 18.59 5.41 20 17.96 7.46 20 9.5V4h-5.5zm.33 9.41l-1.41 1.41 3.13 3.13L14.5 20H20v-5.5l-2.04 2.04-3.13-3.13z" /></svg>
```

## ui/icons/timer.svg

```xml
<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M15 1H9v2h6V1zm-4 13h2V8h-2v6zm8.03-6.61l1.42-1.42c-.43-.51-.9-.99-1.41-1.41l-1.42 1.42A8.962 8.962 0 0 0 12 4c-4.97 0-9 4.03-9 9s4.02 9 9 9 9-4.03 9-9c0-2.12-.74-4.07-1.97-5.61zM12 20c-3.87 0-7-3.13-7-7s3.13-7 7-7 7 3.13 7 7-3.13 7-7 7z" /></svg>
```

