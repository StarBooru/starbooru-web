# Testi dell'interfaccia di StarBooru, italiano.

app-name = StarBooru

## Shared

notfound-title = Qui non c'è niente
notfound-body = Non esiste una pagina { $path }.
notfound-back = Torna alla galleria
notice-close = Chiudi

## Navigation

nav-main = Principale
nav-gallery = Galleria
nav-explore = Esplora
nav-boorus = Booru
nav-tags = Tag
nav-upload = Carica
nav-settings = Impostazioni
nav-settings-tab = Opzioni
search-placeholder = Cerca tag, ad es. cat rating:safe
search-label = Cerca tag
search-clear = Cancella la ricerca
search-suggestions = Tag suggeriti
search-hint = { $tag }, { $count ->
    [one] 1 post
   *[other] { $count } post
}
search-corrected = Ortografia corretta: { $tags }
search-exact = Cerca esattamente come scritto

## La demo della casella di ricerca vuota

demo-tag = Post con questo tag
demo-tag-all = Post con tutti i tag elencati
demo-tag-not = Il meno esclude un tag
demo-figure = Una persona li ha tutti
demo-rating = Solo questa classificazione
demo-rating-any = Una di queste classificazioni
demo-rating-not = Tutto tranne l'esplicito
demo-type = Solo video
demo-date = Di marzo 2024
demo-date-range = Dal 2023 alla fine del 2024
demo-date-age = Di meno di 7 giorni fa
demo-size = Larghi più di 1920 pixel
demo-ratio = Larghezza su altezza 16:9
demo-color = Ha questo colore
demo-text = Questa parola è scritta nell'immagine
demo-order-score = Prima il punteggio più alto
demo-order-random = Un ordine nuovo ogni volta
demo-combined = Tutto insieme

## Page titles

post-number = Post #{ $id }
trash-title = Cestino
dup-title = Trova duplicati
help-title = Guida alla ricerca
admin-title = Amministrazione
privacy-title = Informativa sulla privacy
terms-title = Termini di servizio

## Errors

error-session-expired = La sessione è scaduta. Accedi di nuovo.
error-unreachable = Impossibile raggiungere il server { $server }. Controlla l'indirizzo e la connessione, poi riprova.
error-bad-address = Questo non è l'indirizzo di un server. Scrivine uno come https://booru.example.com.
error-not-a-server = { $server } risponde, ma non è un server StarBooru. Controlla l'indirizzo.
error-rate-limited = Troppe richieste. Riprova più tardi.
error-rate-limited-minutes =
    { $minutes ->
        [one] Troppi tentativi. Riprova tra un minuto.
       *[other] Troppi tentativi. Riprova tra { $minutes } minuti.
    }

## Sign-in

login-tagline = La tua raccolta privata di immagini
login-server-version = { $name } · v{ $version }
login-first-user = Questo server non ha ancora account. L'account che crei ora diventa l'amministratore.
login-server = Server
login-server-placeholder = https://booru.example.com
login-username = Nome utente
login-password = Password
login-wait = Attendi…
login-create-admin = Crea l'account amministratore
login-create = Crea account
login-sign-in = Accedi
login-have-account = Ho già un account
login-want-account = Crea un account
login-wrong = Nome utente o password errati.
login-weak-password = Questa password è facile da indovinare. Prova con una più lunga, o con qualche parola che non c'entra con le altre.
login-or = oppure
login-with = Accedi con { $provider }
password-strength-weak = Facile da indovinare
password-strength-fair = Potrebbe essere più sicura
password-strength-strong = Password sicura
oidc-finishing = Accesso in corso…
oidc-denied = L'accesso è stato annullato.
oidc-failed = L'accesso non è riuscito. Riprova o chiedi all'amministratore.
oidc-expired = Questo accesso è scaduto. Ricomincia.
legal-nav = Note legali
legal-privacy-short = Privacy
legal-terms-short = Termini

## Settings

settings-account = Account
settings-signed-in-as = Accesso come
settings-server-version = Versione del server
settings-sign-out = Esci
settings-library = Raccolta
settings-trash-note = Ripristina o elimina definitivamente i post
settings-dup-note = Trova i post simili e tieni la copia migliore
settings-appearance = Aspetto
settings-theme = Tema
theme-system = Sistema
theme-light = Chiaro
theme-dark = Scuro
settings-language = Lingua
settings-language-system = Sistema
lang-english = English
lang-italian = Italiano
settings-accent = Colore principale
settings-accent-custom = Colore principale personalizzato
accent-no-hue = I grigi non hanno una tonalità da cui ricavare i colori. Scegli un colore più deciso.
accent-violet = Viola
accent-pink = Rosa
accent-coral = Corallo
accent-amber = Ambra
accent-green = Verde
accent-teal = Verde acqua
accent-blue = Blu
accent-red = Rosso
settings-hand = Mano
settings-hand-hint = Mette la navigazione laterale su tablet e portatili, gli strumenti dell'editor e i pulsanti della pagina Carica dal lato della mano che usi.
hand-either = Entrambe
hand-left = Sinistra
hand-right = Destra
settings-saver = Risparmio batteria
settings-saver-hint = Quando è attivo, i video non partono da soli, il post successivo non viene caricato in anticipo e le animazioni sono ridotte. Automatico lo attiva quando la batteria è al 20% o meno e non è in carica, o quando il dispositivo risparmia dati.
settings-saver-now = Attivo ora: la batteria è scarica o il dispositivo sta risparmiando dati.
saver-auto = Automatico
saver-always = Sempre
saver-never = Mai
settings-effects = Effetti
settings-effects-hint = Ridotti mostra superfici piene al posto del vetro smerigliato, e lascia fuori l'entrata delle nuove miniature e il bagliore dietro l'immagine. Automatico lo fa quando lo scorrimento va a scatti su questo dispositivo.
settings-effects-now = Ridotti ora: lo scorrimento andava a scatti su questo dispositivo. Scegli di nuovo Automatico per misurarlo un'altra volta.
effects-auto = Automatico
effects-full = Completi
effects-lite = Ridotti
settings-originals = Apri sempre gli originali nel visualizzatore
settings-originals-hint = Altrimenti le immagini grandi si aprono come copia più leggera, più veloce e con meno dati. L’originale si apre comunque quando ingrandisci, o quando lo schermo ha più pixel della copia.
settings-flush = Togli gli spazi e gli angoli arrotondati tra le immagini
settings-flush-hint = Le miniature stanno una accanto all’altra, con gli angoli squadrati.
settings-keep-original = Tieni l'originale quando salvi una modifica
settings-keep-original-hint = L'immagine o il video modificato si salva come nuovo post accanto all'originale. Altrimenti l'originale va nel cestino.
settings-gestures = Gesti
settings-gestures-hint = Scorri una miniatura di lato per agire subito. Tieni premuto una miniatura o un tag, o fai clic destro, per tutto ciò che puoi farci.
settings-swipe-left = Scorri a sinistra
settings-swipe-right = Scorri a destra
swipe-off = Niente
swipe-delete = Sposta nel cestino
admin-server-title = Server
settings-admin-note = Statistiche, account e impostazioni del server
settings-help-legal = Aiuto e note legali
settings-help-note = Sintassi delle ricerche, esempi e scorciatoie da tastiera
settings-privacy-note = Cosa viene salvato, dove e per quanto tempo
settings-terms-note = Le regole per usare questo server
settings-role = Ruolo
role-admin = amministratore
role-user = utente
settings-storage = Caricamenti
usage-of = { $used } su { $quota }
signin-title = Modi di accesso
signin-password-set = Impostata
signin-password-none = Non impostata
signin-provider = Accesso unico
signin-times = Collegato { $linked } · ultimo uso { $used }
signin-linked = Collegato { $linked }
signin-last = È il tuo unico modo di accedere. Imposta una password prima di scollegarlo.
signin-link = Collega un account { $provider }
signin-unlink = Scollega
signin-unlink-named = Scollega { $name }
oidc-back-settings = Torna alle Impostazioni
password-current = Password attuale
password-new = Nuova password
password-too-short = Almeno { $count } caratteri.
password-repeat = Ripeti la nuova password
password-mismatch = Le password non coincidono.
password-wrong-current = La password attuale è sbagliata.
password-changed = Password cambiata. Gli altri tuoi dispositivi sono stati disconnessi.
password-changing = Modifica in corso…
password-change = Cambia password
password-none = Questo account non ha ancora una password. Impostane una per accedere anche senza il provider.
password-set = Imposta password
sessions-title = Dispositivi connessi
sessions-unknown = Client sconosciuto
sessions-this-device = Questo dispositivo
sessions-times = Attivo { $active } · accesso { $created }
sessions-revoke-others = Esci da tutti gli altri dispositivi
sessions-revoke = Disconnetti questo dispositivo
keys-title = Chiavi API
keys-lead = Permetti a script e altri strumenti di usare StarBooru a tuo nome. Una chiave non può cambiare la password, gli accessi o le chiavi.
keys-copy-now = Copia ora la chiave “{ $name }”. Non verrà più mostrata.
keys-token = Nuova chiave API
keys-copy = Copia
keys-copied = Copiata
keys-copy-failed = Impossibile copiare la chiave. È selezionata, quindi copiala dalla tastiera o dal menu.
keys-none = Ancora nessuna chiave API.
keys-read-only = Sola lettura
keys-full = Accesso completo
keys-expired = Scaduta
keys-used = Usata { $when }
keys-never-used = Mai usata
keys-no-expiry = Non scade
keys-expired-on = Scaduta il { $date }
keys-expires-on = Scade il { $date }
keys-name = Nome
keys-name-placeholder = Chi la userà, ad es. script di backup
keys-access = Accesso
keys-lifetime = Scadenza
keys-never-expires = Mai
keys-days =
    { $count ->
        [one] 1 giorno
       *[other] { $count } giorni
    }
keys-create = Crea chiave
keys-delete = Rimuovi la chiave
keys-delete-named = Rimuovi la chiave { $name }
keys-error-name = Dai un nome alla chiave.
keys-error-long = Il nome supera i { $max } caratteri.
keys-error-control = Il nome non può contenere caratteri di controllo.
keys-error-days = Una chiave può durare da 1 a { $max } giorni.
about-title = Informazioni su StarBooru
about-made-by = Creato da
about-made-by-name = FilipNone
about-icons = Icone
about-icons-names = MingCute, Material Symbols, Solar, Fluent UI
about-lettering = Lettere del logo
about-lettering-name = M PLUS Rounded 1c
about-version = Versione dell'app
about-github = GitHub
about-credits = Tutti i crediti
about-credits-note = Tutti gli autori delle opere usate da StarBooru, con le licenze

## Gallery

retry = Riprova
loading = Caricamento
load-more = Carica altri
no-matches = Nessun risultato
post-alt = Post { $id }
badge-gif = GIF
gallery-tag-info = Informazioni su questo tag
gallery-syntax = Sintassi di ricerca
gallery-to-top = Torna all'inizio
gallery-empty = Ancora nessun post
gallery-empty-body = Carica immagini o video per iniziare la tua raccolta.
gallery-try-fewer = Nessun risultato per questa ricerca. Prova con meno tag.
sort = Ordina
sort-by = Ordina per
sort-new = Più recenti
sort-old = Meno recenti
sort-score = Punteggio più alto
sort-updated = Modificati di recente
sort-size = File più grandi
sort-size-asc = File più piccoli
sort-pixels = Risoluzione più alta
sort-longside = Lato più lungo
sort-tags = Più tag
sort-tags-asc = Meno tag
sort-random = Casuale
view-title = Disposizione della galleria
view-shape = Forma delle miniature
view-square = Quadrate
view-landscape = Orizzontali
view-portrait = Verticali
view-staggered = Sfalsate
view-staggered-by = Forme sfalsate
view-everything = Tutte
view-columns = { $count ->
    [one] 1 colonna
   *[other] { $count } colonne
}
view-per-row = Circa { $count } per riga
view-columns-auto = Automatico
view-flush = Togli gli spazi e gli angoli arrotondati tra le immagini
color-search = Cerca per colore
color-wheel = Scegli un colore dalla ruota
color-hex = Colore esadecimale
color-hex-apply = Usa questo colore
color-hex-invalid = Usa 3 o 6 cifre esadecimali, ad es. #2060e0
color-clear = Qualsiasi colore
color-red = Rosso
color-orange = Arancione
color-yellow = Giallo
color-green = Verde
color-teal = Verde acqua
color-blue = Blu
color-purple = Viola
color-pink = Rosa
color-brown = Marrone
color-black = Nero
color-gray = Grigio
color-white = Bianco

search-save = Salva questa ricerca
search-forget = Dimentica questa ricerca
search-save-failed = Impossibile salvare la ricerca: { $reason }
search-forget-failed = Impossibile dimenticare la ricerca: { $reason }
saved-searches = Ricerche salvate
saved-remove = Rimuovi “{ $name }”
saved-removed = “{ $name }” rimossa

gallery-select = Seleziona i post
gallery-stop-selecting = Termina la selezione
bulk-label = Modifica i post selezionati
bulk-selected =
    { $count ->
        [one] 1 selezionato
       *[other] { $count } selezionati
    }
bulk-select-all = Seleziona tutto
bulk-select-none = Deseleziona tutto
bulk-edit-title = Modifica tag e classificazione
bulk-sheet-title =
    { $count ->
        [one] Modifica 1 post
       *[other] Modifica { $count } post
    }
bulk-add-tags = Aggiungi questi tag
bulk-remove-tags = Togli questi tag
bulk-keep-rating = Mantieni la classificazione
bulk-apply = Applica
bulk-tag-both = “{ $tag }” è in entrambe le liste. Lascialo in una sola.
bulk-nothing = Aggiungi o togli un tag, oppure scegli una classificazione.
bulk-delete-failed = Impossibile spostarli nel cestino: { $reason }
bulk-restore-failed =
    { $count ->
        [one] Impossibile recuperare 1 post: { $reason }
       *[other] Impossibile recuperare { $count } post: { $reason }
    }
bulk-updated =
    { $count ->
        [one] Aggiornato 1 post
       *[other] Aggiornati { $count } post
    }
bulk-moved =
    { $count ->
        [one] Spostato 1 post nel cestino
       *[other] Spostati { $count } post nel cestino
    }
bulk-skipped =
    { $count ->
        [one] saltato 1 post
       *[other] saltati { $count } post
    }
bulk-skipped-undeletable =
    { $count ->
        [one] saltato 1 post che non puoi eliminare
       *[other] saltati { $count } post che non puoi eliminare
    }
bulk-done = { $done }.
bulk-done-skipped = { $done }; { $skipped }.

## Esplora

explore-nothing = Ancora niente da esplorare
explore-nothing-body = Carica immagini o video e compariranno qui.
explore-everything = Esplora tutto
nothing-matches = Nessun risultato per “{ $query }”.
explore-on-this-day = Accadde oggi
explore-no-memories = Nessun post aggiunto in questa data negli anni passati.
explore-years-ago = { $count ->
    [one] 1 anno fa
   *[other] { $count } anni fa
}
explore-random = Scelti a caso
explore-shuffle = Mescola
explore-filter-placeholder = Restringi, ad es. -rating:e
explore-filter-label = Restringi con una ricerca

## Cestino

trash-lead = I post eliminati restano qui finché non li ripristini o li elimini definitivamente. L'eliminazione definitiva rimuove anche il file dal server.
trash-purge-after =
    { $days ->
        [one] Il server elimina definitivamente i post qui un giorno dopo che sono stati spostati nel cestino.
       *[other] Il server elimina definitivamente i post qui { $days } giorni dopo che sono stati spostati nel cestino.
    }
trash-purge-never = I post restano qui finché qualcuno non li elimina definitivamente.
trash-none = Il cestino è vuoto
trash-restore-post = Ripristina il post { $id }
trash-restore-all = Ripristina tutto
trash-restore-all-confirm = Ripristinare tutto?
trash-restored =
    { $count ->
        [one] 1 post ripristinato.
       *[other] { $count } post ripristinati.
    }
trash-restore-failed = Impossibile ripristinare il post { $id }: { $reason }
trash-purge = Elimina definitivamente
trash-purge-post = Elimina definitivamente il post { $id }
trash-purge-failed = Impossibile eliminare definitivamente il post { $id }: { $reason }
trash-purged =
    { $count ->
        [one] 1 post eliminato definitivamente.
       *[other] { $count } post eliminati definitivamente.
    }
trash-empty = Svuota il cestino
trash-empty-confirm = Eliminare tutto definitivamente?
sure = Sicuro?

## Duplicati

dup-matching = Somiglianza
dup-strict = Rigida
dup-normal = Normale
dup-loose = Ampia
dup-lead = Post che si somigliano, prima i più simili. Tieni la copia migliore e le altre finiscono nel cestino.
dup-unhashed =
    { $count ->
        [one] 1 post non può ancora essere confrontato: non ha una miniatura o la sua impronta è in calcolo.
       *[other] { $count } post non possono ancora essere confrontati: non hanno una miniatura o la loro impronta è in calcolo.
    }
dup-none = Nessun duplicato trovato
dup-none-try-loose = Prova la somiglianza Ampia per trovare le copie modificate.
dup-none-loose = Qui niente si somiglia.
dup-groups =
    { $count ->
        [one] 1 gruppo
       *[other] { $count } gruppi
    }
posts-count =
    { $count ->
        [one] 1 post
       *[other] { $count } post
    }
dup-some-of = { $shown } di { $count } post
dup-keep-which = Post da tenere
dup-merge = Copia i loro tag sul post tenuto
dup-dismiss = Non sono duplicati
dup-resolve =
    { $count ->
        [one] Tienine uno, cestina l'altro
       *[other] Tienine uno, cestina gli altri { $count }
    }
dup-unknown-size = Dimensioni sconosciute
dup-keep = Tieni
dup-identical = Identici
dup-nearly-identical = Quasi identici
dup-very-similar = Molto simili
dup-similar = Simili
tags-count =
    { $count ->
        [one] 1 tag
       *[other] { $count } tag
    }
dup-added = Aggiunto il { $date }
dup-post = #{ $id }
dup-unmerge-failed = Non è stato possibile togliere i tag copiati dal post #{ $id }: { $reason }

## Viewer

post-missing-title = Nessun post { $id }
post-missing-body = Potrebbe essere stato eliminato definitivamente, oppure l'indirizzo è sbagliato.
post-back = Indietro
post-prev-title = Precedente (←)
post-prev = Post precedente
post-next-title = Successivo (→)
post-next = Post successivo
zoom-out-title = Riduci (−)
zoom-out = Riduci
zoom-fit = Adatta
zoom-fit-title = Adatta (0)
zoom-reset = Ripristina lo zoom
zoom-in-title = Ingrandisci (+)
zoom-in = Ingrandisci
fullscreen-exit-title = Esci da schermo intero (F)
fullscreen-exit = Esci da schermo intero
fullscreen-title = Schermo intero (F)
fullscreen = Schermo intero
post-actions = Azioni sul post
post-upvote = Voto positivo
post-downvote = Voto negativo
post-votes = Voti
post-score = Punteggio
vote-failed = Impossibile salvare il voto: { $reason }
post-download = Scarica
download-failed = Il download non è finito. Controlla la connessione e riprova.
share-button = Condividi
editor-open = Modifica immagine o video
editor-image-short = Modifica immagine
editor-video-short = Taglia
editor-title-image = Modifica immagine
editor-title-video = Taglia video
editor-tools = Strumenti
editor-crop = Ritaglia
editor-trim = Taglia
editor-history = Cronologia
editor-undo = Annulla
editor-redo = Ripeti
editor-undo-keys = Annulla (Ctrl+Z)
editor-redo-keys = Ripeti (Ctrl+Maiusc+Z)
editor-reset = Ricomincia
editor-save = Salva
editor-save-copy = Salva copia
editor-save-keys = Salva come nuovo post (Ctrl+S)
editor-save-options = Cosa succede all'originale
editor-save-replace = Sostituisci l'originale
editor-save-replace-hint = L'originale va nel cestino
editor-keep-original = Tieni l'originale
editor-keep-original-hint = La modifica si salva come nuovo post accanto all'originale
editor-saved = Salvato come nuovo post
editor-saved-binned = Originale spostato nel cestino
editor-undo-save = Annulla
editor-ratio = Proporzioni
editor-ratio-free = Libero
editor-ratio-original = Originale
editor-ratio-custom = Personale
editor-ratio-swap = Verticale o orizzontale
editor-ratio-width = Larghezza
editor-ratio-height = Altezza
editor-straighten = Raddrizza
editor-straighten-reset = Torna a 0°
editor-turn-flip = Ruota e rifletti
editor-turn-left = Ruota a sinistra
editor-turn-right = Ruota a destra
editor-flip-h = Rifletti da sinistra a destra
editor-flip-v = Rifletti dall'alto in basso
editor-crop-area = Ritaglio. Le frecce lo spostano, e con Maiusc di più.
editor-handle-n = Bordo in alto
editor-handle-s = Bordo in basso
editor-handle-w = Bordo a sinistra
editor-handle-e = Bordo a destra
editor-handle-nw = Angolo in alto a sinistra
editor-handle-ne = Angolo in alto a destra
editor-handle-sw = Angolo in basso a sinistra
editor-handle-se = Angolo in basso a destra
editor-timeline = Parte da tenere
editor-start = Inizio (secondi)
editor-end = Fine (secondi)
editor-now = Usa il punto attuale
editor-now-title = Prendi il punto in cui si trova il video
editor-keeps = Tiene { $length } di { $total }.
editor-keyframe-hint = Dove il video lo permette la qualità originale resta intatta; altrimenti la clip viene ricodificata, e può volerci un po'.
editor-bad-time = Inserisci un inizio prima della fine, entro la durata del video.
editor-too-short = La clip deve durare almeno un decimo di secondo.
editor-no-change = Fai prima una modifica.
post-delete = Elimina
post-delete-title = Sposta nel cestino
press-deleted = Spostato nel cestino
press-undo = Annulla
post-in-trash = Questo post è nel cestino.
trash-restore = Ripristina
post-tags = Tag
post-edit = Modifica
post-edit-tags = Modifica i tag
post-no-tags = Ancora nessun tag.
post-edit-title = Modifica tag e classificazione (E)
post-tags-hint = Metti artist:, character:, copyright: o meta: davanti a un tag per sceglierne la categoria.
post-did-you-mean = Intendevi?
post-replace-tag = Sostituisci { $from } con { $to }
tag-offers = Tag che corrispondono
tag-error-prefix = “{ $tag }” non può essere un tag: un tag non può iniziare con - o ~.
tag-error-long = “{ $tag }” è troppo lungo per un tag. Resta entro { $max } caratteri.
tag-error-many = Un post può avere al massimo { $max } tag.
tag-error-other = “{ $tag }” non può essere un tag.
cancel = Annulla
save = Salva
ai-suggest = Suggerisci tag
ai-suggesting = Chiedo all'IA…
ai-add-all = Aggiungi tutti
ai-new-tag = nuovo
ai-no-suggestions = Nessun suggerimento per questo post.
ai-rating = Classificazione suggerita: { $rating }
category-general = Generale
category-artist = Artista
category-character = Personaggio
category-copyright = Copyright
category-meta = Meta
section-people = Persone
section-hair = Capelli
section-eyes = Occhi
section-skin = Pelle
section-body = Corpo
section-expression = Espressione
section-pose = Posa e inquadratura
section-clothing = Abbigliamento
section-setting = Ambientazione
section-style = Stile
section-other = Altro
people-title = Persone
people-none = Nessuno è ancora segnato. Aggiungendo le persone nell'immagine tagghi vestiti e aspetto di ognuna, e 1girl, 2boys e solo restano giusti.
people-hint = I loro tag vanno anche sul post. I conteggi come 1girl e solo seguono le persone. Separa i tag con spazi.
people-add = Aggiungi una persona
people-edit = Modifica
people-edit-label = Modifica le persone
people-remove = Togli questa persona
people-kind = Chi
people-numbered = { $kind } { $n }
people-character = Personaggio
people-cosplay = In cosplay da
people-as = come
people-summary-character = { $person }: { $character }
people-summary-cosplay = { $person }, in cosplay da { $cosplay }
people-summary-both = { $person }: { $character } come { $cosplay }
people-clothing = Vestiti
people-looks = Aspetto
people-error-many = Un post può avere al massimo { $max } persone.
people-error-many-tags = { $person } ha troppi tag. Una persona può averne al massimo { $max }.
slot-head = Testa
slot-body = Busto
slot-bottom = Sotto e gambe
slot-shoes = Scarpe
slot-wear = Altri vestiti
slot-shape = Corporatura
slot-eyes = Occhi
slot-skin = Pelle
slot-hair = Capelli
slot-pose = Posa
slot-other = Altro
ai-suggest-people = Suggerisci persone
ai-no-people = L'IA non ha trovato persone.
post-details = Dettagli
rating = Classificazione
rating-safe = sicuro
rating-sensitive = delicato
rating-questionable = discutibile
rating-explicit = esplicito
post-size = Dimensione
post-dimensions = Risoluzione
post-length = Durata
post-type = Tipo
post-file-date = Data del file
post-colors = Colori
post-color-search = Cerca { $color }
post-added = Aggiunto
post-source = Fonte
post-sample-note = Stai vedendo una copia più leggera. Ingrandendo si carica l’originale.
post-show-original = Mostra originale
post-related = Simili
press-open = Apri
press-open-tab = Apri in una nuova scheda
swipe-like = Mi piace
press-unlike = Togli il mi piace
press-liked = Ti piace
press-unliked = Mi piace tolto
delete-failed = Impossibile spostarlo nel cestino: { $reason }
swipe-select = Seleziona

## Tag

tags-sort = Ordina i tag
tags-most-used = Più usati
tags-a-z = A–Z
tags-filter-label = Filtra i tag
tags-filter-placeholder = Filtra per nome o alias
tags-any-category = Tutti
tags-none-yet = Ancora nessun tag
tags-none-match = Nessun tag corrisponde
tags-appear = I tag compaiono qui quando i post li usano.
tags-missing = Nessun tag chiamato “{ $name }”
tags-all = Tutti i tag
tags-browse =
    { $count ->
        [one] Sfoglia 1 post
       *[other] Sfoglia { $count } post
    }
tags-is-alias = “{ $name }” è un alias di questo tag.
tags-category = Categoria
tags-tree = Posizione nell'albero
tags-tree-hint = Cercare un tag trova anche i tag sotto di esso, e la cima dell'albero è la sezione in cui vengono raggruppati i tag di un post.
tags-tree-top = In cima: niente sopra.
tags-parent = Tag genitore
tags-move = Sposta sotto
tags-move-top = Porta in cima
tags-children = Sotto di esso
tags-children-none = Ancora niente sotto.
tags-aliases = Alias
tags-aliases-hint = Altri nomi per questo tag. Taggare o cercare con un alias usa questo tag. Aggiungere un alias che è già un tag lo unisce a questo.
tags-aliases-none = Nessun alias.
tags-implies = Implica
tags-implies-hint = I post con questo tag ricevono anche questi tag, compresi quelli taggati in precedenza.
tags-implies-none = Non implica altri tag.
tags-implied-by = Implicato da
tags-implied-by-hint = Tag che aggiungono questo.
tags-implied-by-none = Nessun tag implica questo.
tags-new-alias = Nuovo alias
tags-implied-tag = Tag implicato
tags-remove = Rimuovi
tags-remove-named = Rimuovi { $name }
tags-admins-only = Solo gli amministratori possono modificarlo.
tags-merge-confirm =
    { $count ->
        [one] “{ $name }” è già un tag, su 1 post. Come alias viene unito a questo tag, e non si può annullare.
       *[other] “{ $name }” è già un tag, su { $count } post. Come alias viene unito a questo tag, e non si può annullare.
    }
tags-merge = Unisci
add = Aggiungi
tag-menu-search = Cerca questo tag
tag-menu-add = Aggiungi alla ricerca
tag-menu-exclude = Escludilo
tag-menu-details = Dettagli del tag
tag-menu-copy = Copia il nome
tag-copied = Nome del tag copiato
tag-copy-failed = Non è stato possibile copiare il nome del tag

## Caricamento

upload-files = Scegli i file
upload-folder = Scegli una cartella
upload-folder-tags = Tagga i file con il nome della loro cartella
upload-autotag = Fai aggiungere i tag all'AI
upload-link = Aggiungi un link
upload-clear = Svuota l'elenco
upload-drop = Trascina o incolla qui file o link
upload-drop-now = Rilascia per caricare
upload-urls-placeholder = Incolla link a file o a post di un booru, uno per riga
upload-urls-label = Link a file o a post di un booru
upload-urls-only = Inserisci indirizzi che iniziano con http:// o https://.
upload-skipped =
    { $more ->
        [0] Si accettano solo immagini e video. Esclusi: { $names }.
        [one] Si accettano solo immagini e video. Esclusi: { $names } e un altro.
       *[other] Si accettano solo immagini e video. Esclusi: { $names } e altri { $more }.
    }
upload-paste-nothing = Non c'era nessun file o link da incollare.
upload-shared-tags = Tag per tutti i file
upload-source = Fonte
upload-source-placeholder = Fonte (facoltativa)
upload-source-hint = Da dove vengono i file. Un link è la fonte di sé stesso.
upload-start =
    { $count ->
        [0] Carica
        [one] Carica 1 file
       *[other] Carica { $count } file
    }
upload-progress = { $done ->
        [one] 1 di { $count } completato
       *[other] { $done } di { $count } completati
    }
upload-queued = In coda
upload-uploading = In caricamento
upload-uploaded = Caricato
upload-stopped = Annullato
upload-failed = Non riuscito
upload-known = Già presente
upload-known-post = Già presente come post #{ $id }
upload-open-post = Apri il post #{ $id }
upload-too-big = Più grande di quanto accetta questo server (al massimo { $max }).
upload-unsupported = Questo server accetta solo un'immagine o un video in un formato che conosce.
upload-unreadable = Impossibile leggere il file: { $error }
upload-gone = Il file non è più disponibile. Aggiungilo di nuovo.
upload-file-tags-placeholder = Tag solo per questo file
upload-file-tags-label = Tag per { $name }
upload-remove = Rimuovi { $name }
upload-cancel = Interrompi il caricamento di { $name }
upload-retry = Riprova { $name }
upload-open = Apri { $name }
upload-close = Chiudi
upload-remove-item = Togli dal caricamento
upload-open-link = Apri il link
upload-rating-usual = Come gli altri ({ $rating })
upload-done = Importato
upload-job-running = In importazione
upload-job-cancel = Annulla
activity-pending =
    { $count ->
        [one] 1 caricamento o importazione in corso
       *[other] { $count } caricamenti e importazioni in corso
    }
activity-uploaded =
    { $count ->
        [one] 1 file caricato
       *[other] { $count } file caricati
    }
activity-imported =
    { $count ->
        [one] 1 post importato
       *[other] { $count } post importati
    }
activity-known =
    { $count ->
        [one] 1 già presente
       *[other] { $count } già presenti
    }
activity-failed =
    { $count ->
        [one] 1 non riuscito
       *[other] { $count } non riusciti
    }

## Guida alla ricerca

help-try = Prova una ricerca
help-try-label = Ricerca da spiegare
help-try-placeholder = cat -dog rating:safe order:score
help-search = Cerca
help-search-for = Cerca { $term }
help-tags = Tag
help-tag = ha il tag
help-tag-all = ha tutti i tag elencati
help-tag-not = non ha il tag
help-tag-tree = ha footwear o un tag sotto di esso, come boots o sneakers
help-figure = una persona nell'immagine ha tutti questi tag; girl, boy o other dice chi. Un meno davanti trova i post dove nessuno li ha
help-tag-category = il tag some_name; quando tagghi, il prefisso lo rende un tag artista (anche character:, copyright: e meta:)
help-tag-spaces = due tag: gli spazi separano i termini, quindi scrivi blue_sky
help-tag-meta = post ad alta risoluzione senza audio. Il server aggiunge da solo tag meta come lowres, highres, absurdres, wide_image, tall_image, hd, full_hd, 4k, 8k, horizontal_image, vertical_image, square_image, animated, video, sound e long_video
help-filters = Filtri
help-rating = classificato sicuro, delicato (s), discutibile (q) o esplicito (e)
help-rating-any = una qualsiasi delle classificazioni elencate
help-rating-not = qualsiasi classificazione tranne esplicito
help-type = immagini statiche; anche type:animation e type:video
help-deleted = post nel cestino
help-date = la data nel file, o quella di caricamento se non ne ha; anche date:2024 e date:2024-03-15
help-date-range = dall’inizio del primo periodo alla fine dell’ultimo; date:2023.. e date:..2024 lasciano aperto un estremo
help-date-compare = dopo quel periodo; anche >=, < e <=
help-date-age = con più di 30 giorni; date:<7d ha meno di 7 giorni. Anche h, w, mo e y
help-size = più largo di 1920 pixel; anche height:, >=, <, <=, un valore esatto width:1920 e un intervallo width:1000..2000
help-mpixels = più di 8 megapixel; mpixels:8 da solo vuol dire circa 8
help-ratio = larghezza su altezza, come 16:9 o 1.78, con margine di 0,01; ratio:1 è quadrato e ratio:>1 è più largo che alto
help-size-not = qualsiasi dimensione tranne queste; un post di dimensione ignota resta
help-color = ha questo colore o, se nessun post ce l'ha, il più vicino salvato; anche orange, yellow, green, teal, blue, purple, pink, brown, black, gray e white
help-color-hex = qualsiasi colore, in esadecimale; anche #2060e0 e 26e
help-color-not = non ha questo colore
help-exact = di solito un tag sconosciuto viene corretto con il più vicino in uso; così i tag si cercano esattamente come scritti
help-text = Testo nelle immagini
help-text-word = il testo letto nell’immagine contiene la parola (maiuscole o minuscole)
help-text-phrase = contiene tutte queste parole; le virgolette le tengono insieme
help-text-not = il testo letto nell’immagine non contiene la parola
help-sorting = Ordinamento
help-order-new = prima i più recenti (predefinito); order:old per i più vecchi
help-order-score = prima il punteggio più alto
help-order-size = prima i file più grandi
help-order-pixels = prima la risoluzione più alta; anche order:mpixels
help-order-longside = prima il lato più lungo
help-order-tags = prima quelli con più tag
help-order-updated = prima quelli modificati più di recente
help-order-direction = aggiungi _asc o _desc per scegliere la direzione
help-order-random = un ordine casuale nuovo ogni volta
help-combined = Combinazioni
help-combined-cats = gatti adatti a tutti senza cani, prima i migliori
help-combined-videos = i video più grandi
help-tips = Buono a sapersi
help-tip-normalise = I tag sono in minuscolo, e spazi o trattini bassi ripetuti diventano un solo _.
help-tip-aliases = Un alias trova il suo tag, e i tag implicati vengono aggiunti quando tagghi un post.
help-tip-terms = Una ricerca contiene al massimo 40 termini.
help-shortcuts = Scorciatoie da tastiera e gesti
key-prev-next = post precedente o successivo
key-edit = modifica tag e classificazione
key-fullscreen = schermo intero
key-zoom = ingrandisci o riduci
key-fit = adatta allo schermo
key-escape = esci dallo schermo intero, poi torna alla galleria
key-double-click = Doppio clic
key-double-click-action = ingrandisci un punto, di nuovo per adattare
key-swipe = Scorri a sinistra o a destra
key-swipe-action = post precedente o successivo
people-girl = Ragazza
people-boy = Ragazzo
people-other = Altro
explain-deleted = Nel cestino
explain-tagged = Con il tag { $tag }
explain-tagged-all = Con tutti i tag { $tags }
explain-not-tagged = Senza il tag { $tag }
explain-tagged-none = Senza nessuno dei tag { $tags }
explain-rated = Classificati { $ratings }
explain-not-rated = Non classificati { $ratings }
list-or = { $rest } o { $last }
explain-images = Solo immagini statiche
explain-animations = Solo animazioni
explain-videos = Solo video
explain-figure = Una persona ha tutti questi tag: { $tags }
explain-no-figure = Nessuno ha tutti questi tag: { $tags }
explain-figure-kind = Una persona ({ $kind }) ha tutti questi tag: { $tags }
explain-no-figure-kind = Nessuna persona ({ $kind }) ha tutti questi tag: { $tags }
explain-figure-only = Una persona segnata come { $kind }
explain-no-figure-only = Nessuna persona segnata come { $kind }
explain-date = Data { $value }
explain-not-date = Data diversa da { $value }
explain-size = { $field }: { $value }
explain-not-size = { $field }: non { $value }
explain-color = Ha il colore { $value }, o il più vicino salvato
explain-not-color = Non ha il colore { $value }
explain-exact = I tag si cercano esattamente come scritti
explain-size-width = Larghezza
explain-size-height = Altezza
explain-size-mpixels = Megapixel
explain-size-ratio = Proporzioni
explain-text = Il testo nell’immagine contiene “{ $text }”
explain-not-text = Il testo nell’immagine non contiene “{ $text }”
explain-every = Tutti i post
explain-order-new = Prima i più recenti
explain-order-old = Prima i più vecchi
explain-order-random = In ordine casuale
explain-order-score-desc = Per punteggio, prima il più alto
explain-order-score-asc = Per punteggio, prima il più basso
explain-order-size-desc = Per dimensione del file, prima la più grande
explain-order-size-asc = Per dimensione del file, prima la più piccola
explain-order-pixels-desc = Per risoluzione, prima la più alta
explain-order-pixels-asc = Per risoluzione, prima la più bassa
explain-order-longside-desc = Per lato più lungo, prima il più lungo
explain-order-longside-asc = Per lato più lungo, prima il più corto
explain-order-tags-desc = Per numero di tag, prima i più numerosi
explain-order-tags-asc = Per numero di tag, prima i meno numerosi
explain-order-updated-desc = Per ultima modifica, prima la più recente
explain-order-updated-asc = Per ultima modifica, prima la meno recente

## Informativa sulla privacy e termini di servizio

legal-back-sign-in = Torna all’accesso
legal-back-settings = Torna alle Impostazioni
legal-server = Per { $name } all’indirizzo { $address }
legal-server-address = Per il server all’indirizzo { $address }
legal-section-link = Link a questa sezione
privacy-lead = StarBooru è un software che gira su un server scelto da te o da qualcuno di cui ti fidi. Niente di ciò che salvi viene inviato al progetto StarBooru, né ad altri, se non come descritto in questa pagina.
privacy-responsible = Chi è responsabile
privacy-responsible-body = La persona o l'organizzazione che gestisce questo server (il gestore) decide chi può registrarsi ed è responsabile dei dati che contiene. Le domande sui tuoi dati vanno rivolte a lei.
privacy-server = Cosa salva il server
privacy-server-account = Il tuo account: nome utente, ruolo e un hash Argon2id con sale della tua password. La password in sé non viene mai salvata. Se accedi tramite il provider di identità del gestore, il server conserva l'identificativo che quel provider ti assegna e il nome utente o l'email che riporta, per riconoscerti la volta successiva.
privacy-server-sessions = Le tue sessioni: un hash di ogni token di accesso, il tipo di dispositivo da cui proviene (ad esempio “Firefox on Linux” o “StarBooru app on Android”), quando è stata creata e usata l'ultima volta, e quando scade. Le chiavi API che crei sono conservate allo stesso modo: un hash della chiave, i primi caratteri per distinguerla, il suo nome e quando è stata usata l'ultima volta.
privacy-server-files = I tuoi caricamenti: ogni file con dimensione, risoluzione, checksum e un'impronta visiva per la ricerca dei duplicati, e ciò che il server ne ricava: la data contenuta nel file, i colori principali e l'eventuale testo in un'immagine statica. Il server registra chi ha caricato ogni post.
privacy-server-content = Ciò che aggiungi: tag, le persone segnate in un post, classificazioni, fonti, traduzioni delle pagine di fumetti, voti con il momento in cui li hai dati, e ricerche salvate.
privacy-server-shares = Per ogni link di condivisione che crei: i post che contiene, il titolo, la scadenza, e quante volte e quando è stato aperto l'ultima volta (non da chi).
privacy-server-sites = Per gli altri siti: le ricerche che salvi su altri booru, gli account che salvi per loro (il nome utente come l'hai scritto, la chiave cifrata con il segreto del server), gli indirizzi che importi, tenuti 7 giorni dopo la fine, e ogni importazione di raccolte con i suoi conteggi finché non la rimuovi.
privacy-server-limits = Per rallentare i tentativi di indovinare le password, il server conta gli accessi falliti per account e per indirizzo di rete, e le registrazioni per indirizzo. Questi conteggi restano in memoria al massimo un'ora e non vengono mai scritti su disco.
privacy-device = Cosa resta sul tuo dispositivo
privacy-device-storage = Questa app tiene una sola voce nella memoria locale del browser: l'indirizzo del server, il tuo token di accesso, il tuo nome utente e ruolo, le tue preferenze (come il tema, la lingua, l'aspetto della galleria e i booru aggiunti per indirizzo) e il punto in cui era la galleria nelle tue ultime 30 ricerche.
privacy-device-sign-in = Mentre accedi tramite un provider di identità, la scheda tiene un codice monouso nella sua memoria di sessione, rimosso quando torni.
privacy-device-sign-out = Uscendo vengono rimossi il token, il nome utente e il ruolo, e le ricerche ricordate dalla galleria. L'indirizzo del server e le preferenze restano finché non cancelli i dati di questo sito nel browser.
privacy-device-cache = StarBooru non usa cookie. Immagini e video finiscono nella cache del browser, come su qualsiasi sito, e tutto il resto che l'app carica resta nella memoria della scheda finché non la chiudi.
privacy-not-done = Cosa StarBooru non fa
privacy-not-done-tracking = Niente statistiche d'uso, pubblicità o tracciamento, e nessun servizio di terze parti oltre a un provider di identità o a un fornitore di IA che il gestore sceglie di configurare. Questa app carica codice, stili e icone solo da questo server.
privacy-not-done-contacts = Il server contatta altri siti solo quando cerchi, sfogli o importi da un altro booru, importi da un URL o accedi tramite il provider di identità del gestore, e quando un amministratore installa un modello di IA locale, che scarica da dove è pubblicato. Le sue richieste si presentano come StarBooru e non portano nessuno dei tuoi dati, tranne l'account che hai salvato per quel sito, che va solo a quel sito.
privacy-not-done-boorus = Quando sfogli un altro booru qui, il browser carica le immagini direttamente da quel sito, senza dire quale pagina le ha chieste, quindi il sito vede il tuo indirizzo di rete come lo vedrebbe sulle sue pagine. Le immagini dei siti che le mostrano solo sulle proprie pagine passano invece da questo server.
privacy-not-done-ai = Se un amministratore configura un fornitore di IA, gli vengono inviate le miniature dei post per suggerire i tag (anche dei nuovi caricamenti, quando il tagging automatico è attivo), il testo letto in un’immagine quando lo traduci, e l’immagine stessa quando se ne traducono le vignette di fumetti o manga, a mano o per i nuovi caricamenti quando la traduzione automatica è attiva. Un fornitore sulla rete del gestore le tiene lì; i modelli locali girano sul server e niente ne esce.
privacy-not-done-library = Mentre si importa un’intera raccolta, la chiave che hai dato per quel sito resta sul server, e in ogni backup fatto nel frattempo; viene cancellata quando l’importazione finisce.
privacy-not-done-logs = Il contenuto dei file, il testo letto nelle immagini, le password e i token non vengono mai scritti nei log del server.
privacy-who-sees = Chi può vedere i tuoi post
privacy-who-sees-body = Chiunque abbia un account su questo server può vedere i post che contiene, e scaricarne i file originali con tutto ciò che contengono, come il luogo in cui è stata scattata una foto. Un post mostra il suo punteggio, non chi l'ha votato. I link ai file forniti all'app sono firmati e scadono, quindi un link copiato smette di funzionare entro 12 ore. Se crei un link di condivisione, chiunque lo abbia può vedere i post che contiene, senza i loro tag, finché non scade o non lo rimuovi.
privacy-keeping = Conservazione ed eliminazione
privacy-keeping-bin = I post eliminati finiscono nel cestino. Da lì un amministratore, o chi li ha caricati, può cancellarli insieme ai file, e il gestore può far svuotare il cestino da solo dopo un certo numero di giorni.
privacy-keeping-sessions = Le sessioni terminano quando esci, dopo 90 giorni, o quando disconnetti gli altri dispositivi dalle Impostazioni. Cambiare la password disconnette tutti gli altri dispositivi; le chiavi API continuano a funzionare finché non le rimuovi o scadono.
privacy-keeping-admins = Gli amministratori di questo server vedono di ogni account il nome utente, il ruolo, quando si è iscritto ed è stato attivo l'ultima volta, e quanto ha caricato. Possono impostare un limite di spazio, reimpostare una password dimenticata, disattivare un account o eliminarlo. Per eliminare il tuo account, chiedi a uno di loro: ne verranno rimossi le sessioni, le chiavi, i voti e le ricerche salvate, mentre i suoi post restano.
privacy-keeping-backups = Gli amministratori possono fare backup del database, a mano o a intervalli regolari. Un backup contiene tutto quanto elencato sopra tranne i file, quindi ciò che viene cancellato qui resta nei backup precedenti finché non vengono eliminati. Il gestore può conservare altre copie, con i suoi tempi.
terms-lead = Questi termini regolano l'uso di questo server StarBooru. Il gestore che lo amministra può aggiungerne altri.
terms-account = Il tuo account
terms-account-keys = Non condividere la tua password né le tue chiavi API, e disconnetti i dispositivi che non usi più (Impostazioni › Dispositivi connessi).
terms-account-admins = Il primo account di un server ne è l'amministratore. Gli amministratori possono gestire tag, alias e implicazioni, svuotare il cestino, cambiare le impostazioni del server, configurare gli strumenti di IA, fare backup e gestire gli account: ruoli, limiti di spazio, reimpostazione delle password, disattivazione ed eliminazione.
terms-content = I tuoi contenuti
terms-content-rights = Carica e importa solo ciò che hai il diritto di conservare e condividere con le persone di questo server e con chiunque riceva un tuo link di condivisione, e ciò che è legale dove si trova il server.
terms-content-yours = Mantieni ogni diritto su ciò che carichi. Consenti al server di conservarlo, crearne miniature e anteprime, e mostrarlo agli account di questo server e alle persone con cui lo condividi.
terms-content-originals = Gli originali sono conservati byte per byte. Classifica i post con onestà, così i filtri per classificazione funzionano per tutti.
terms-fair = Uso corretto
terms-fair-access = Non tentare di accedere ad account o dati altrui, né di aggirare i limiti del server.
terms-fair-load = Non sovraccaricare il server, né i siti che contatta per te, con richieste automatiche.
terms-warranty = Nessuna garanzia
terms-warranty-body = StarBooru è fornito così com'è, senza garanzie di alcun tipo. Tieni una tua copia di tutto ciò che non puoi permetterti di perdere. Il gestore può sospendere gli account che violano questi termini e può modificarli; la versione in vigore è sempre su questa pagina.

## Altri booru

explore-collection = Raccolta
boorus-search = Cerca nei booru
boorus-sites = Siti
boorus-site = Sito
boorus-site-remove = Rimuovi { $site }
boorus-address = Indirizzo
boorus-api = Tipo di sito
boorus-account = Account
boorus-account-forget = Dimentica
boorus-account-again = Inseriscilo di nuovo
boorus-login = Nome utente
boorus-user-id = ID utente
boorus-key = Chiave API
boorus-password-hash = Hash della password
boorus-dedupe = Nascondi i duplicati
boorus-known = Già presente
boorus-extra-tags = Tag in più
boorus-save-label = Salva i post selezionati
boorus-save = Salva { $count }
boorus-open-site = Apri sul sito
boorus-also = Anche su
boorus-score = Punteggio
boorus-bad-address = Inserisci l’indirizzo del sito, come https://booru.example.
boorus-need-account = Inserisci sia il nome utente (o l’ID utente) sia la chiave.
boorus-too-long = È più lungo di quanto un sito accetti.
boorus-saving = Salvataggio…

## Importazioni di raccolte

library-title = Importa da un altro booru
library-lead = Sposta qui un’intera raccolta da szurubooru, Shimmie2, Danbooru, Stash, Blombooru, Monbooru o LANBooru: ogni post con tag, classificazione, fonte, data e i tuoi “mi piace”, più alias, implicazioni, alberi di tag, e pool o album come tag. Il lavoro lo fa il tuo server, quindi puoi chiudere l’app. Importando di nuovo si aggiunge ciò che è nuovo, e un post che hai già riceve solo i tag. Alias e implicazioni arrivano solo se importa un amministratore. La tua chiave resta sul server mentre l’importazione è in corso e viene cancellata quando finisce.
library-left-out = Restano fuori: commenti, altri utenti e i loro account, note sulle immagini, titoli e descrizioni, punteggi del sito e l’ordine interno dei pool. StarBooru non ha un posto per questi dati.
library-source = Importa da
library-hint-szurubooru = Accedi con il tuo nome utente e un token dalla scheda Login tokens del tuo account. Senza, ottieni ciò che vedono gli ospiti.
library-hint-shimmie2 = Serve Ouroboros API attivata nell’Extension Manager di Shimmie, e una chiave API se gli ospiti non vedono tutto. I suoi alias e le regole di auto-tag arrivano se sono pubblici, tranne quelli di più parole. I pool non si possono leggere: Shimmie non ha un’API per questi.
library-hint-danbooru = Il tuo nome utente e la chiave API dal profilo. Senza, ottieni ciò che vedono gli ospiti e i tuoi preferiti restano fuori. Sui siti grandi sono milioni di post, quindi usalo sulla tua istanza.
library-hint-stash = Scene e immagini, dalle più vecchie. La chiave API è in Settings › Security. Stash non ha classificazioni di sicurezza, quindi ogni post riceve quella che scegli qui sotto. Gli interpreti diventano tag personaggio, gli studi tag artista, e un voto da 60 in su diventa un “mi piace”.
library-hint-blombooru = La chiave API del tuo account. Per alias e implicazioni serve la chiave di un amministratore di quel sito; le implicazioni basate su schemi di tag non arrivano.
library-hint-monbooru = Un token con accesso in lettura, se il sito li usa. Arrivano tutte le gallerie del server. I suoi alias non si possono leggere: Monbooru non ha un’API per questi.
library-hint-lanbooru = Nessun account: le letture di LANBooru sono aperte. Non ha alias, implicazioni o album.
library-unrated = Classificazione dei post senza un tag che la definisca
library-check = Controlla
library-checking = Controllo…
library-start = Importa tutto
library-found =
    { $count ->
        [one] Trovato 1 post.
       *[other] Trovati { $count } post.
    }
library-found-some = Il sito risponde, ma non dice quanti post ha.
library-no-rules = Alias e implicazioni resteranno fuori, perché qui solo un amministratore può importarli.
library-full = Possono attendere o essere in corso fino a cinque importazioni. Avvia questa quando un’altra finisce.
library-bad-address = Questo indirizzo non è valido. Scrivilo come http://192.168.1.4:8080.
library-need-login = Questo sito richiede il nome utente insieme alla chiave.
library-need-key = Questo sito richiede la chiave insieme al nome utente.
library-list = Importazioni
library-state-queued = In attesa
library-state-rules = Lettura di alias e implicazioni
library-state-posts = Importazione dei post
library-state-groups = Lettura di pool e “mi piace”
library-state-done = Finita
library-state-stopped = Fermata
library-state-failed = Non riuscita
library-count =
    { $done ->
        [one] 1 fatto
       *[other] { $done } fatti
    }
library-count-of = { $done } di { $total }
library-count-merged =
    { $count ->
        [one] 1 già presente
       *[other] { $count } già presenti
    }
library-count-failed =
    { $count ->
        [one] 1 non riuscito
       *[other] { $count } non riusciti
    }
library-count-rules =
    { $count ->
        [one] 1 alias o implicazione
       *[other] { $count } alias e implicazioni
    }
library-stop = Ferma
library-retry = Importa di nuovo
library-remove = Togli dall’elenco
library-show-failed = Mostra i non riusciti
library-account-again = La chiave è stata cancellata alla fine di questa importazione. Inseriscila di nuovo, o lascia vuoto per leggere ciò che vedono gli ospiti.
ago-now = adesso
ago-minutes =
    { $count ->
        [one] 1 minuto fa
       *[other] { $count } minuti fa
    }
ago-hours =
    { $count ->
        [one] 1 ora fa
       *[other] { $count } ore fa
    }
ago-yesterday = ieri
ago-days =
    { $count ->
        [one] 1 giorno fa
       *[other] { $count } giorni fa
    }

## Share links

share-title =
    { $count ->
        [one] Condividi questo post
       *[other] Condividi { $count } post
    }
share-lead =
    { $count ->
        [one] Chi ha il link può vedere questo post senza un account. Tag, voti e i tuoi dati restano nascosti.
       *[other] Chi ha il link può vedere questi post senza un account. Tag, voti e i tuoi dati restano nascosti.
    }
share-name = Titolo (facoltativo)
share-name-placeholder = es. Foto delle vacanze
share-lifetime = Il link funziona per
share-until-removed = Finché non lo rimuovi
share-days =
    { $count ->
        [one] 1 giorno
       *[other] { $count } giorni
    }
share-create = Crea link
share-ready = Link pronto. Chiunque lo abbia può aprirlo.
share-link = Link di condivisione
share-copy = Copia link
share-copied = Copiato
share-copy-failed = Non è stato possibile copiare il link. È selezionato: copialo dalla tastiera o dal menu.
share-native = Condividi con…
share-done = Fatto
share-close = Chiudi
share-posts =
    { $count ->
        [one] 1 post
       *[other] { $count } post
    }
share-error-none = Scegli almeno un post da condividere.
share-error-many = Un link può contenere al massimo { $max } post.
share-error-title-long = Il titolo può avere al massimo { $max } caratteri.
share-error-title-control = Il titolo non può contenere tabulazioni o a capo.
share-error-days = Un link funziona da 1 a { $max } giorni, o finché non lo rimuovi.
shares-title = Link condivisi
shares-lead = I link creati con Condividi. Rimuoverne uno lo disattiva; un file già aperto da qualcuno può continuare a caricarsi per qualche ora.
shares-none = Ancora nessun link. Usa Condividi su un post, o seleziona dei post nella galleria.
shares-untitled = Link senza titolo
shares-made = Creato { $when }
shares-views =
    { $count ->
        [0] Mai aperto
        [one] Aperto una volta
       *[other] Aperto { $count } volte
    }
shares-expired = Scaduto
shares-no-expiry = Non scade
shares-expired-on = Scaduto il { $date }
shares-expires-on = Scade il { $date }
shares-expires-minutes =
    { $count ->
        [one] Scade tra 1 minuto
       *[other] Scade tra { $count } minuti
    }
shares-expires-hours =
    { $count ->
        [one] Scade tra 1 ora
       *[other] Scade tra { $count } ore
    }
shares-expires-days =
    { $count ->
        [one] Scade tra 1 giorno
       *[other] Scade tra { $count } giorni
    }
shares-delete = Rimuovi link
shares-delete-named = Rimuovi il link { $name }
shared-missing-title = Questo link non funziona
shared-missing-body = Potrebbe essere scaduto, o chi l'ha condiviso l'ha rimosso.
shared-from = Condiviso da { $server }
shared-until = disponibile fino al { $date }
shared-empty = Qui non c'è più niente.
shared-open = Elemento { $n }
shared-previous = Precedente
shared-next = Successivo

## Administration

admin-only = Solo gli amministratori possono aprire questa pagina.
admin-tab-overview = Panoramica
admin-tab-accounts = Account
admin-tab-backups = Backup
admin-tab-server = Server
admin-on = Attivo
admin-off = Disattivato
admin-posts = Post
admin-images = Immagini
admin-animations = Animazioni
admin-videos = Video
admin-bin = Nel cestino
admin-tags = Tag
admin-accounts = Account
admin-media = Media su disco
admin-database = Dimensione del database
admin-uploads-title = Caricamenti, ultimi 30 giorni
admin-uploads-total =
    { $count ->
        [one] 1 caricamento negli ultimi 30 giorni
       *[other] { $count } caricamenti negli ultimi 30 giorni
    }
admin-running = Avviato
admin-upload-limit = Caricamento massimo
admin-bin-size = Dimensione del cestino
admin-disabled-accounts = Account disattivati
admin-video-thumbs = Miniature video (ffmpeg)
admin-url-import = Importazione da URL
admin-accounts-lead = Tutti gli account di questo server, cosa hanno caricato e quando sono stati attivi l'ultima volta.
admin-disabled = Disattivato
admin-you = Tu
admin-active = attivo { $when }
admin-never-active = mai attivo
admin-joined = iscritto il { $date }
admin-manage = Gestisci
admin-manage-named = Gestisci { $name }
admin-role-saved = Ruolo cambiato.
admin-limit = Limite di spazio
admin-limit-none = Nessun limite
admin-limit-hint = In GB, oppure con l'unità, come 500 MB. I caricamenti che lo supererebbero vengono rifiutati. Lascia vuoto per nessun limite.
admin-limit-save = Imposta
admin-limit-saved = Limite di spazio salvato.
admin-limit-invalid = Scrivi una dimensione, come 5 GB o 500 MB, o lascia vuoto.
admin-password-new = Nuova password
admin-password-hint = Per chi ha dimenticato la propria. Lo disconnette ovunque; le sue chiavi API continuano a funzionare.
admin-password-reset = Reimposta
admin-password-reset-done = Password reimpostata. Comunica quella nuova.
admin-disable = Disattiva account
admin-enable = Riattiva account
admin-disabled-done = Account disattivato e disconnesso.
admin-enabled-done = Account riattivato.
admin-delete = Elimina account
admin-delete-confirm = Eliminare { $name } definitivamente?
admin-delete-note = L'eliminazione rimuove l'account, i suoi voti, le ricerche salvate e le chiavi. I suoi post restano.
admin-deleted = Account { $name } eliminato.
admin-server-lead = Si applicano subito e hanno la precedenza sulle variabili d'ambiente del server.
admin-name = Nome del server
admin-name-hint = Mostrato nella pagina di accesso.
admin-name-error = Dai al server un nome da 1 a { $max } caratteri.
admin-registration = Chiunque può creare un account
admin-registration-hint = Disattivato: possono accedere solo gli account esistenti.
admin-trash-days = Svuota il cestino dopo (giorni)
admin-trash-days-hint = I post nel cestino più vecchi di così vengono eliminati definitivamente, ogni ora. 0 li tiene finché non lo svuoti a mano.
admin-trash-days-error = Scrivi un numero di giorni da 0 a { $max }.
admin-auto-tags = Aggiungi ai nuovi post i tag di ciò che mostra il file
admin-auto-tags-hint = Tag meta come highres, lowres, wide_image, video, sound e long_video. Ogni post viene taggato una volta sola, quindi un tag che togli resta tolto. Attivandolo vengono taggati anche i post più vecchi al prossimo avvio del server.
admin-save = Salva
admin-saved = Salvato
admin-bin-open = Apri il cestino
admin-bin-empty-confirm =
    { $count ->
        [one] Eliminare 1 post definitivamente?
       *[other] Eliminare { $count } post definitivamente?
    }
admin-ai-title = IA
admin-ai-unavailable = L'IA locale non può avviarsi su questo server.
admin-ai-unavailable-api = Ogni strumento può comunque usare un'API.
admin-ai-hw-threads = { $count } thread
admin-ai-hw-memory = { $total } di RAM
admin-ai-tagging = Suggerimento tag
admin-ai-translation = Traduzione
admin-ai-comics = Traduzione fumetti
admin-ai-mode-off = Spenta
admin-ai-mode-local = Locale
admin-ai-mode-api = API
admin-ai-speed = Velocità
admin-ai-preset-fast = Veloce
admin-ai-preset-balanced = Bilanciata
admin-ai-preset-accurate = Accurata
admin-ai-presets-off = Non disponibile su questa macchina: { $presets }
admin-ai-state-ready = Pronta
admin-ai-state-preparing = Preparazione…
admin-ai-state-downloading = Download { $percent }%
admin-ai-state-failed = Non riuscita
admin-ai-models = Modelli
admin-ai-no-memory = Memoria insufficiente
admin-ai-gpu = GPU
admin-ai-high-accuracy = Alta precisione
admin-ai-in-use = In uso
admin-ai-use = Usa
admin-ai-install = Installa
admin-ai-uninstall = Disinstalla
admin-ai-url = Indirizzo
admin-ai-model = Modello
admin-ai-key = Chiave API
admin-ai-key-set = Salvata
admin-ai-key-none = Nessuna
admin-ai-key-clear = Rimuovi chiave
admin-ai-test = Prova
admin-ai-testing = Prova in corso…
admin-ai-test-ok = Risposta in { $time }
admin-ai-test-failed = L'API non ha risposto: { $reason }.
admin-ai-error-url = Inserisci l'indirizzo dell'API.
admin-ai-error-url-scheme = L'indirizzo inizia con http:// o https://.
admin-ai-error-model = Inserisci il nome del modello da usare.
admin-ai-error-long = Indirizzo, modello e chiave possono avere al massimo { $max } caratteri ciascuno, su una riga.
admin-ai-autotag = Tagga in automatico i nuovi caricamenti
admin-ai-auto-translate = Traduci i nuovi fumetti in
admin-ai-auto-translate-hint = Codice della lingua, come it. Vuoto: no.
admin-ai-translate-invalid = Non è un codice di lingua. Scrivine uno come it o pt-BR, o lascia vuoto.
backups-title = Backup del database
backups-lead = Un backup contiene il database: post, tag, account e impostazioni. I file multimediali restano nella cartella dei dati, quindi copia anche quella.
backups-unavailable = Questo server non può fare backup: manca il client PostgreSQL (pg_dump). Installalo, oppure indica il percorso con STARBOORU_PG_DUMP.
backups-now = Fai un backup ora
backups-running = Backup in corso…
backups-made = Backup fatto.
backups-none = Ancora nessun backup.
backups-automatic = Automatico
backups-download-named = Scarica il backup { $name }
backups-delete = Elimina backup
backups-delete-named = Elimina il backup { $name }
backups-schedule-title = Backup automatici
backups-every = Ogni (giorni)
backups-every-hint = 0 disattiva i backup automatici.
backups-every-error = Da 0 a { $max } giorni.
backups-keep = Da tenere
backups-keep-hint = I backup automatici più vecchi vengono rimossi. Quelli fatti a mano restano finché non li elimini.
backups-keep-error = Da 1 a { $max }.
backups-restore-title = Ripristino
backups-restore = Ferma StarBooru, poi carica il backup nel suo database con pg_restore:
backups-restore-note = Poi riavvia StarBooru. La guida al deployment ha tutti i passaggi.

## Dimensioni, e i ripensamenti del visualizzatore e delle pagine dei tag

size-b = { $size } B
size-kb = { $size } KB
size-mb = { $size } MB
size-gb = { $size } GB
size-tb = { $size } TB
sure-named = Sicuro? { $action }
post-unsaved = Prima salva o annulla le modifiche.
post-neighbours-failed = Impossibile trovare i post vicini a questo: { $reason }
post-media-failed = Il file non si è caricato. Il suo link potrebbe essere scaduto.
people-remove-named = Togli { $person }
ai-suggest-people-replace = Sostituire le persone inserite?

## Errori a parole: lato server, ricerche, ricerche salvate, download e caricamenti

error-server-fault = Qualcosa è andato storto su { $server }. Riprova tra un minuto. Se continua a succedere, avvisa chi gestisce il server: i dettagli sono nel suo log.
error-storage-full = Lo spazio su disco di { $server } è finito. Chi lo gestisce deve liberarne un po' prima che si possa salvare altro.
error-proxy-too-large = { $server } l'ha rifiutato come troppo grande prima che arrivasse a StarBooru. Chi lo gestisce può alzare il limite di caricamento del server web che gli sta davanti.
error-unexpected-answer = { $server } ha dato una risposta che StarBooru non si aspettava ({ $status }). Controlla l'indirizzo o riprova più tardi.
query-error-terms = Troppi termini di ricerca. Usane al massimo { $max }.
query-error-rating = Classificazione “{ $value }” sconosciuta. Usa g, s, q o e.
query-error-kind = Tipo “{ $value }” sconosciuto. Usa image, animation o video.
query-error-status = Stato “{ $value }” sconosciuto. Usa active o deleted.
query-error-order = Ordine “{ $value }” sconosciuto. Usa new, old, score, size, pixels, longside, tags, updated o random, e aggiungi _asc per invertirlo.
query-error-text-empty = text: richiede qualche parola, come text:ciao o text:"due parole".
query-error-text-long = text: è troppo lungo. Resta entro { $max } caratteri.
query-error-figure-empty = { "{}" } richiede dei tag all'interno, come { "{" }girl blonde_hair boots{ "}" }.
query-error-figure-kinds = Un termine { "{…}" } indica due tipi. Usane uno solo tra girl, boy e other.
query-error-date = date:{ $value } non è una data. Prova date:2024-03, date:2023..2024 o date:>30d.
query-error-size = { $field }:{ $value } non è una dimensione. Prova width:>1920, mpixels:>8 o ratio:16:9.
query-error-color = color:{ $value } non è un colore. Prova color:red o color:2060e0.
query-error-tag-long = Un tag nella ricerca è troppo lungo. Resta entro { $max } caratteri per tag.
query-error-tag-other = Un tag nella ricerca contiene un carattere che nessun tag può avere.
saved-error-no-name = Dai un nome alla ricerca.
saved-error-name-long = Il nome è troppo lungo. Resta entro { $max } caratteri.
saved-error-control = Il nome contiene un carattere non ammesso.
saved-error-empty = Non c'è nessuna ricerca da salvare.
saved-error-long = La ricerca è troppo lunga per essere salvata. Resta entro { $max } caratteri.
download-expired = Il link a questo file è scaduto. Ricarica la pagina e riprova.
upload-stopped-late = { $name } era già arrivato, come post { $id }.
upload-drop-waiting =
    { $count ->
        [one] Il file aspetta nella pagina Carica.
       *[other] { $count } file aspettano nella pagina Carica.
    }

## Other boorus, Settings and the administration, second pass

boorus-saved-as = Salvato qui come post #{ $id }
boorus-save-failed = Non è stato possibile salvarlo: { $reason }
boorus-post-missing = Questo indirizzo non dice quale post aprire.
boorus-account-left = Sul server è salvato un account per { $site } ({ $login }), ma il sito non è in questo elenco.
search-list-failed = Non è stato possibile leggere le ricerche salvate: { $reason } Premi per riprovare.
keys-done = Fatto
admin-delete-named = Elimina l’account { $name }
admin-ai-uninstall-named = Disinstalla { $name }
admin-ai-downloading-named = Download di { $name }
