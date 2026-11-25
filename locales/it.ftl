# Testi dell'interfaccia di StarBooru, italiano.

app-name = StarBooru

## Shared

notfound-title = Qui non c'è niente
notfound-body = Non esiste una pagina { $path }.
notfound-back = Torna alla galleria
page-back = Indietro
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
nav-forum = Forum
account-button = Il tuo account, { $name }
account-title = Il tuo account
account-guest-title = Senza un account
account-guest-hint = Vota, carica, scrivi nel forum
account-see-profile = Vedi il tuo profilo
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

## Profili

profile-title = Profilo
profile-edit = Modifica profilo
profile-done = Fatto
profile-stat-posts = Post
profile-stat-forum = Messaggi nel forum
profile-stat-joined = Iscritto il
profile-missing = Nessuno si chiama { $name } qui
profile-missing-body = Controlla il nome, o forse l'account è stato eliminato.
profile-off = Nessun profilo su questo server
profile-off-body = Questo server è privato: non mostra il profilo di nessuno.
profile-display-name = Nome visualizzato
profile-display-name-hint = Quello che gli altri vedono al posto del tuo nome utente, che resta il tuo identificativo. Lascialo vuoto per mostrare il nome utente.
profile-about = Su di te
profile-chars-left =
    { $count ->
        [one] Resta 1 carattere
       *[other] Restano { $count } caratteri
    }
profile-chars-over =
    { $count ->
        [one] 1 carattere di troppo
       *[other] { $count } caratteri di troppo
    }
profile-save = Salva profilo
profile-name-long = Un nome visualizzato ha al massimo { $max } caratteri.
profile-about-long = Questo testo ha al massimo { $max } caratteri.
profile-bad-character = Togli i caratteri invisibili o di controllo.
profile-picture-add = Aggiungi un'immagine
profile-picture-change = Cambia immagine
profile-picture-remove = Togli immagine
profile-picture-removed = Immagine tolta.
profile-picture-saved = Immagine salvata.
profile-picture-hint = JPEG, PNG, WebP o GIF, fino a 5 MB. Poi scegli il cerchio.
profile-picture-use = Usa questa immagine
profile-picture-sending = Salvataggio…
profile-picture-big = L'immagine supera { $max }. Scegline una più piccola.
profile-picture-kind = Non è un'immagine JPEG, PNG, WebP o GIF.
profile-picture-small = L'immagine ha un lato più corto di { $min } pixel.
profile-picture-outside = Quel quadrato non sta dentro l'immagine.
profile-crop-label = Trascina l'immagine per sistemarla nel cerchio, anche con le frecce. La rotellina, due dita o + e − ingrandiscono.
profile-crop-zoom = Zoom
profile-crop-in = Ingrandisci
profile-crop-out = Rimpicciolisci

## Il forum

forum-off = Nessun forum su questo server
forum-off-body = Questo server è privato, e solo uno pubblico ha un forum.
forum-search = Cerca nel forum
forum-search-category = Cerca in questa categoria
forum-found = Discussioni con “{ $q }”
forum-found-clear = Mostra tutte
forum-found-none = Nessuna discussione contiene tutte queste parole.
forum-no-categories = Ancora nessuna categoria.
forum-threads =
    { $count ->
        [one] 1 discussione
       *[other] { $count } discussioni
    }
forum-replies =
    { $count ->
        [0] Nessuna risposta
        [one] 1 risposta
       *[other] { $count } risposte
    }
forum-no-activity = Ancora niente
forum-threads-none = Ancora nessuna discussione qui. Inizia la prima.
forum-more = Mostra altre
forum-more-posts = Mostra altri messaggi
forum-back = Torna al forum
forum-category-missing = Questa categoria non esiste
forum-thread-missing = Questa discussione non esiste
forum-pinned = In evidenza
forum-locked = Chiusa
forum-author-gone = Account eliminato
forum-thread-new = Nuova discussione
forum-thread-title = Titolo
forum-thread-body = Primo messaggio
forum-thread-post = Pubblica discussione
forum-guest-why = Accedi per scrivere nel forum. Tornerai qui.
forum-guest-reply = Accedi per rispondere in questa discussione.
forum-write = Scrivi
forum-preview = Anteprima
forum-preview-empty = Ancora niente da mostrare.
forum-placeholder = Testo semplice, con Markdown: **grassetto**, *corsivo*, elenchi, > citazioni. >>123 rimanda al post 123.
forum-sending = Pubblicazione…
forum-reply = Rispondi
forum-reply-send = Pubblica risposta
forum-wait =
    { $seconds ->
        [one] Pubblico tra 1 secondo…
       *[other] Pubblico tra { $seconds } secondi…
    }
forum-wait-cancel = Non pubblicare
forum-slow-down = Stai scrivendo troppo in fretta. Riprova tra un attimo.
forum-slow-down-seconds =
    { $seconds ->
        [one] Potrai scrivere di nuovo tra 1 secondo.
       *[other] Potrai scrivere di nuovo tra { $seconds } secondi.
    }
forum-slow-down-minutes =
    { $minutes ->
        [one] Oggi hai scritto { $posts } messaggi. Potrai scrivere di nuovo tra 1 minuto.
       *[other] Oggi hai scritto { $posts } messaggi. Potrai scrivere di nuovo tra { $minutes } minuti.
    }
forum-slow-down-hours =
    { $hours ->
        [one] Oggi hai scritto { $posts } messaggi. Potrai scrivere di nuovo tra 1 ora.
       *[other] Oggi hai scritto { $posts } messaggi. Potrai scrivere di nuovo tra { $hours } ore.
    }
forum-locked-note = Questa discussione è chiusa. Nessuno può rispondere.
forum-edited = modificato
forum-edited-at = Modificato { $when }
forum-edit = Modifica
forum-delete = Elimina messaggio
forum-edited-done = Messaggio modificato.
forum-post-deleted = Messaggio eliminato.
forum-post-gone = Questo messaggio è stato eliminato.
forum-undone = Annullato.
forum-thread-manage = Gestisci discussione
forum-pin = Metti in evidenza
forum-pin-hint = Prima nella sua categoria
forum-unpin = Togli dall'evidenza
forum-unpin-hint = Di nuovo tra le altre
forum-lock = Chiudi
forum-lock-hint = Rispondono solo gli admin
forum-unlock = Riapri
forum-unlock-hint = Tutti possono rispondere
forum-pinned-done = Discussione messa in evidenza.
forum-unpinned-done = Discussione tolta dall'evidenza.
forum-locked-done = Discussione chiusa.
forum-unlocked-done = Discussione riaperta.
forum-rename = Rinomina
forum-renamed-done = Discussione rinominata.
forum-move-to = Sposta in { $name }
forum-moved-done = Spostata in { $name }.
forum-thread-delete = Elimina discussione
forum-thread-delete-sure =
    { $count ->
        [one] Eliminare definitivamente questa discussione e il suo messaggio?
       *[other] Eliminare definitivamente questa discussione e i suoi { $count } messaggi?
    }
forum-thread-deleted = Discussione eliminata.
forum-title-empty = Dai un titolo alla discussione.
forum-title-long = Un titolo ha al massimo { $max } caratteri.
forum-body-empty = Prima scrivi qualcosa.
forum-body-long = Un messaggio ha al massimo { $max } caratteri.
forum-category-new = Nuova categoria
forum-category-name = Nome
forum-category-description = A cosa serve
forum-category-add = Aggiungi categoria
forum-category-edit = Modifica categoria
forum-category-edit-named = Modifica { $name }
forum-category-place = Posizione nell'elenco
forum-category-up = Sposta su
forum-category-down = Sposta giù
forum-category-delete = Elimina categoria
forum-category-deleted = Categoria eliminata.
forum-category-not-empty = Sposta o elimina le sue discussioni per eliminarla.
forum-category-name-empty = Dai un nome alla categoria.
forum-category-name-long = Un nome ha al massimo { $max } caratteri.
forum-category-description-long = Una descrizione ha al massimo { $max } caratteri.
mention-post = Post n. { $id }
mention-missing = Questo post non c'è
mention-no-picture = Nessuna immagine
mention-hidden = Nascosto dalla tua blacklist: { $entries }

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
error-slow-down = Stai sfogliando molto in poco tempo per questo server. Rallenta e riprova tra un attimo, o accedi per continuare subito.
error-slow-down-seconds =
    { $seconds ->
        [one] Stai sfogliando molto in poco tempo per questo server. Rallenta e riprova tra 1 secondo, o accedi per continuare subito.
       *[other] Stai sfogliando molto in poco tempo per questo server. Rallenta e riprova tra { $seconds } secondi, o accedi per continuare subito.
    }
error-slow-down-minutes =
    { $minutes ->
        [one] Stai sfogliando molto in poco tempo per questo server. Rallenta e riprova tra 1 minuto, o accedi per continuare subito.
       *[other] Stai sfogliando molto in poco tempo per questo server. Rallenta e riprova tra { $minutes } minuti, o accedi per continuare subito.
    }
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
guest-sign-in = Accedi
guest-sign-up = Crea un account
guest-back = Torna a sfogliare
guest-keep-browsing = Continua a sfogliare senza account
guest-invite-title = Accedi per fare di più
guest-page-needs-account = Questa pagina è per chi ha un account su questo server.
guest-account-note = Vota, carica e salva le ricerche
guest-account-why = Con un account puoi votare, caricare, salvare le ricerche e ritrovare i post che hai aperto.
guest-vote-why = Accedi per votare. Tornerai su questo post.
guest-tag-vote-why = Accedi per votare i tag. Tornerai su questo post.
guest-needs-account = Accedi per vederlo. Questo server lo mostra solo a chi ha un account.
guest-gallery-empty = Qui non è ancora stato pubblicato nulla.
guest-server-closed = Ora questo server è aperto solo a chi ha un account. Accedi per continuare.
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

settings-group-account = Account e sicurezza
settings-group-appearance-note = Tema, colore e lingua
settings-group-viewing = Visualizzazione e gesti
settings-group-viewing-note = Blacklist, miniature, visualizzatore, gesti e mano
settings-group-battery = Batteria ed effetti
settings-group-battery-note = Risparmio batteria ed effetti visivi
settings-group-library-note = Cestino, duplicati e importazioni
settings-group-sharing = Condivisione e chiavi API
settings-group-sharing-note = Link condivisi e chiavi per altri strumenti
settings-group-about = Aiuto e informazioni
settings-group-about-note = Guida, privacy, riconoscimenti e versione
settings-account = Account
settings-signed-in-as = Accesso come
settings-profile = Profilo
settings-server-version = Versione del server
settings-sign-out = Esci
settings-library = Raccolta
settings-trash-note = Ripristina o elimina definitivamente i post
settings-dup-note = Trova i post simili e tieni la copia migliore
settings-people = Persone
settings-people-hint = La scheda Persone nel visualizzatore. Spenta si nasconde, senza cancellare nulla
settings-appearance = Aspetto
settings-theme = Tema
theme-system = Sistema
theme-light = Chiaro
theme-dark = Scuro
settings-oled = Sfondo nero
settings-oled-hint = Risparmia batteria sugli schermi OLED
settings-language = Lingua
settings-language-system = Sistema
lang-english = English
lang-italian = Italiano
settings-accent = Colore principale
settings-accent-custom = Colore principale personalizzato
accent-violet = Viola
accent-pink = Rosa
accent-coral = Corallo
accent-amber = Ambra
accent-green = Verde
accent-teal = Verde acqua
accent-blue = Blu
accent-red = Rosso
settings-hand = Mano
hand-either = Entrambe
hand-left = Sinistra
hand-right = Destra
settings-saver = Risparmio batteria
settings-saver-now = Attivo ora: la batteria è scarica o il dispositivo sta risparmiando dati.
settings-saver-auto = Si attiva con la batteria al 20% o meno e non in carica, o quando il dispositivo risparmia dati
settings-saver-does = Quando è attivo
saver-does-videos = I video aspettano che li avvii tu
saver-does-preload = Il post successivo non si carica in anticipo
saver-does-motion = Le animazioni sono ridotte
saver-auto = Automatico
saver-always = Sempre
saver-never = Mai
settings-effects = Effetti
settings-effects-now = Ridotti ora: lo scorrimento andava a scatti su questo dispositivo. Scegli di nuovo Automatico per misurarlo un'altra volta.
settings-effects-auto = Si riducono se lo scorrimento va a scatti su questo dispositivo
settings-effects-does = Quando sono ridotti
effects-does-glass = Barre piene al posto del vetro smerigliato
effects-does-thumbs = Le nuove miniature compaiono senza entrata
effects-does-glow = Nessun bagliore dietro le immagini
effects-auto = Automatico
effects-full = Completi
effects-lite = Ridotti
blacklist-title = Blacklist
blacklist-lead = I post con questi tag o classificazioni restano fuori dalle ricerche e vengono coperti quando li apri.
blacklist-none = Ancora nessun tag nascosto.
blacklist-add = Aggiungi tag da nascondere
blacklist-remove = Smetti di nascondere { $tag }
blacklist-removed = { $entry } non è più nascosto
blacklist-ratings = Nascondi classificazioni
blacklist-saving = Salvataggio…
blacklist-guest-note = Salvata su questo dispositivo.
blacklist-default = Usa la lista del server
blacklist-error-many = Una blacklist contiene al massimo { $max } voci.
blacklist-error-term = “{ $word }” non è un tag né una classificazione. Una blacklist nasconde tag, e classificazioni come rating:explicit.
settings-tiles = Miniature
settings-tiles-spaced = Con spazi
settings-tiles-flush = Attaccate
settings-viewer = Visualizzatore
settings-originals = Apri sempre gli originali nel visualizzatore
settings-originals-note = Usa più dati. Altrimenti le immagini grandi si aprono più leggere finché non ingrandisci
settings-keep-original = Tieni l'originale quando salvi una modifica
settings-keep-original-note = Altrimenti l'originale va nel cestino
settings-gestures = Gesti
settings-swipe-left = Scorri a sinistra
settings-swipe-right = Scorri a destra
settings-hold-hint = Tieni premuto o fai clic destro su una miniatura o un tag per tutto ciò che puoi farci
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
install-app = Installa StarBooru
install-note = In una finestra tutta sua, con un'icona tra le tue app
install-done = StarBooru è installata: la trovi tra le tue app.
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
sort-rating = Più votati
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
saved-booru-searches = Ricerche salvate sui booru
saved-all = Tutte le ricerche salvate
saved-all-short = Tutte
saved-filter = Filtra le ricerche salvate
saved-none-match = Nessuna ricerca salvata corrisponde.
saved-sort = Ordina le ricerche salvate
saved-sort-recent = Usate di recente
saved-sort-newest = Più recenti
saved-sort-name = Nome
saved-sort-mine = Il mio ordine
search-save-short = Salva ricerca
search-saved = Salvata
saved-remove = Rimuovi
saved-removed = “{ $name }” rimossa
saved-added = Salvata come “{ $name }”
saved-more = Altro per “{ $name }”
saved-rename = Rinomina
saved-name = Nome della ricerca salvata
saved-renamed = Rinominata in “{ $name }”
saved-rename-failed = Impossibile rinominare la ricerca: { $reason }
saved-move-earlier = Sposta prima
saved-move-later = Sposta dopo
saved-drag = Trascina per spostare
saved-moved = “{ $name }” spostata

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
explore-window = Tendenze di
explore-today = Oggi
explore-week = Settimana
explore-month = Mese
explore-year = Anno
explore-phrase-today = oggi
explore-phrase-week = questa settimana
explore-phrase-month = questo mese
explore-phrase-year = quest'anno
explore-phrase-days =
    { $count ->
        [one] nell'ultimo giorno
       *[other] negli ultimi { $count } giorni
    }
explore-trending = Di tendenza
explore-no-likes = Nessun Mi piace { $when }. Metti Mi piace ai post e compariranno qui.
explore-popular-tags = Tag popolari
explore-tags-note = sui post aggiunti { $when }
explore-no-tags = Nessun post con tag aggiunto { $when }.
explore-added = Post aggiunti { $when }
nothing-matches = Nessun risultato per “{ $query }”.
explore-recent = Visti di recente
explore-recent-clear = Svuota
explore-recent-clear-label = Svuota i visti di recente
explore-recent-cleared = Cronologia svuotata
explore-on-this-day = Accadde oggi
explore-no-memories = Nessun post aggiunto in questa data negli anni passati.
explore-years-ago = { $count ->
    [one] 1 anno fa
   *[other] { $count } anni fa
}
explore-random = Scelti a caso
explore-shuffle = Mescola
explore-search = Cerca in Esplora

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
player = Lettore video
player-play = Riproduci
player-play-title = Riproduci (K)
player-pause = Pausa
player-pause-title = Pausa (K)
player-replay = Riproduci di nuovo
player-position = Posizione
player-time-of = { $now } di { $total }
player-mute = Disattiva l'audio
player-mute-title = Disattiva l'audio (M)
player-unmute = Riattiva l'audio
player-unmute-title = Riattiva l'audio (M)
player-volume = Volume
player-volume-level = { $percent }%
player-speed = Velocità di riproduzione
player-speed-now = Velocità di riproduzione: { $speed }
player-speed-value = { $speed }×
player-settings = Velocità, ripetizione e picture in picture
player-loop = Ripeti
player-pip = Picture in picture
player-pip-exit = Esci da picture in picture
player-back = −{ $seconds } s
player-forward = +{ $seconds } s
player-failed = Il video non si è caricato. Il link potrebbe essere scaduto, oppure questo browser non ne supporta il formato.
player-failed-network = Il video ha smesso di caricarsi. Controlla la connessione e riprova.
player-failed-format = Questo browser non riesce a riprodurre il video. Scaricalo per guardarlo con un altro lettore.
post-actions = Azioni sul post
post-upvote = Voto positivo
post-downvote = Voto negativo
post-votes = Voti
post-score = Punteggio
stars-of =
    { $count ->
        [one] 1 stella su 5
       *[other] { $count } stelle su 5
    }
stars-yours = Il tuo voto: { $stars }
stars-none-yours = Non hai ancora votato
stars-summary =
    { $count ->
        [one] Media { $average } su 5, da 1 voto
       *[other] Media { $average } su 5, da { $count } voti
    }
stars-votes =
    { $count ->
        [one] 1 voto
       *[other] { $count } voti
    }
stars-no-votes = Ancora nessun voto
vote-failed = Impossibile salvare il voto: { $reason }
tag-vote-up = { $tag } è giusto qui
tag-vote-down = { $tag } non c'entra
tag-vote-up-undo = Togli il tuo voto a favore di { $tag }
tag-vote-down-undo = Togli il tuo voto contro { $tag }
tag-vote-score = Punteggio { $score }
tag-vote-failed = Impossibile salvare il voto sul tag: { $reason }
tag-voted-off = I voti hanno tolto { $tag } da questo post.
post-download = Scarica
download-failed = Il download non è finito. Controlla la connessione e riprova.
share-button = Condividi
reverse-button = Trova la fonte
reverse-title = Cerca questa immagine su altri siti
reverse-menu = Trova la fonte su
reverse-lens = Google Lens
reverse-iqdb = iqdb
reverse-saucenao = SauceNAO
reverse-trace = trace.moe
reverse-upload = Carica una copia ridotta
reverse-address = Invia il link a una copia ridotta
reverse-paste = La copia, da incollare lì
reverse-save = La salva, da caricare lì
reverse-copied = Immagine copiata. Incollala su { $site } con { $keys }.
reverse-saved = Immagine salvata. Caricala su { $site } dai download.
reverse-copy-refused = Il browser non ha permesso di copiare l'immagine, quindi è stata salvata. Caricala su { $site } dai download.
reverse-blocked = Il browser ha bloccato la nuova scheda.
reverse-open = Apri { $site }
reverse-read-failed = Impossibile preparare l'immagine da inviare: { $reason }
reverse-unreadable = Il browser non è riuscito a leggere l'immagine. Ricarica la pagina e riprova.
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
post-add-as = Aggiungi tag come { $category }
post-add-category = Categoria dei nuovi tag
post-add-suggested = Aggiungi { $name }
post-tags-as-text = Modifica come testo
post-tag-removed = { $name } rimosso
post-tags-added =
    { $count ->
        [one] 1 tag aggiunto
       *[other] { $count } tag aggiunti
    }
post-tags-removed =
    { $count ->
        [one] 1 tag rimosso
       *[other] { $count } tag rimossi
    }
post-tags-unsaved = Non ancora salvato
post-tags-no-changes = Nessuna modifica
post-save-title = Salva (Ctrl+Invio)
cover-title = Nascosto dalla tua blacklist
cover-show = Mostra
cover-edit = Modifica blacklist
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
ai-suggest-short = Suggerisci
ai-suggested = Suggeriti
ai-dismiss = Chiudi
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
people-turned-off = Le persone sono state disattivate nelle Impostazioni. Questa scheda sparisce quando salvi o annulli.
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
post-uploader = Caricato da
post-sample-note = Stai vedendo una copia più leggera. Ingrandendo si carica l’originale.
post-show-original = Mostra originale
post-related = Simili
press-open = Apri
press-open-tab = Apri in una nuova scheda
swipe-like = Mi piace
press-unlike = Togli il mi piace
press-liked = Ti piace
press-unliked = Mi piace tolto
press-star = Dai 5 stelle
press-unstar = Togli le tue stelle
press-starred = 5 stelle date
delete-failed = Impossibile spostarlo nel cestino: { $reason }
swipe-select = Seleziona

## Tag

tags-sort = Ordina i tag
tags-most-used = Più usati
tags-a-z = A–Z
tags-filter-label = Filtra i tag per nome o alias
tags-filter-placeholder = Filtra i tag
tags-show-all = Mostra tutti i tag
tags-posts =
    { $count ->
        [one] 1 post
       *[other] { $shown } post
    }
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
tags-category-of = { $name }: { $category }. Cambia la categoria
tags-change-category = Cambia la categoria
tags-recoloured = Ora { $name } è { $category }
tags-recolour-failed = Impossibile cambiare la categoria: { $reason }
tags-search-named = Mostra i post con il tag { $name }
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
help-order-rating = prima i più votati
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
help-server = Questo server
help-server-private = È privato: solo chi ha un account vede i suoi post. Un link di condivisione mostra i post che scegli a chiunque lo abbia.
help-server-public = È pubblico: chiunque può sfogliare post, tag ed Esplora senza un account. Un post si condivide con il suo indirizzo.
help-votes-likes = Un voto è un Mi piace o un Non mi piace. Premilo di nuovo per toglierlo. order:score mette prima i più apprezzati.
help-votes-stars = Un voto va da 1 a 5 stelle. Premi di nuovo la tua stella per toglierlo. order:score mette prima i più votati, e un post ha bisogno di qualche voto per salire.
help-forum = Il forum è nella barra di navigazione, al posto di Carica. Chiunque può leggerlo; per scrivere serve un account, al massimo una volta ogni 15 secondi. >>123 o post #123 in un messaggio rimandano a quel post.
help-tag-votes = Ogni tag di un post si può votare: le frecce accanto dicono se è giusto lì. Un tag votato abbastanza in basso viene tolto dal post, e solo un amministratore può rimetterlo.
help-profiles = Ognuno ha un profilo con un'immagine, un nome e due parole, che chiunque può vedere. Carica, Opzioni e il tuo profilo sono sotto la tua immagine, accanto alla navigazione.
help-shortcuts = Scorciatoie da tastiera e gesti
key-prev-next = post precedente o successivo
key-edit = modifica tag e classificazione
key-fullscreen = schermo intero
key-zoom = ingrandisci o riduci
key-fit = adatta allo schermo
key-escape = esci dallo schermo intero, poi torna alla galleria
key-space = Spazio
key-play = riproduci o metti in pausa un video
key-seek = un video indietro o avanti di 5 secondi; anche Maiusc con ← →
key-mute = disattiva o riattiva l'audio di un video
key-jump = salta in un video: 0 all'inizio, 5 a metà
key-double-click = Doppio clic
key-double-click-action = ingrandisci un punto, di nuovo per adattare; su un video, schermo intero
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
list-and = { $rest } e { $last }
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
explain-order-rating-desc = Per valutazione, prima la più alta
explain-order-rating-asc = Per valutazione, prima la più bassa
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
privacy-server-profile = Su un server pubblico, ciò che metti nel tuo profilo: un nome visualizzato, qualche parola su di te e un'immagine, conservata in due piccole copie ritagliate sul quadrato che hai scelto. L'immagine che hai caricato non viene conservata.
privacy-server-sessions = Le tue sessioni: un hash di ogni token di accesso, il tipo di dispositivo da cui proviene (ad esempio “Firefox on Linux” o “StarBooru app on Android”), quando è stata creata e usata l'ultima volta, e quando scade. Le chiavi API che crei sono conservate allo stesso modo: un hash della chiave, i primi caratteri per distinguerla, il suo nome e quando è stata usata l'ultima volta.
privacy-server-files = I tuoi caricamenti: ogni file con dimensione, risoluzione, checksum e un'impronta visiva per la ricerca dei duplicati, e ciò che il server ne ricava: la data contenuta nel file, i colori principali e l'eventuale testo in un'immagine statica. Il server registra chi ha caricato ogni post.
privacy-server-content = Ciò che aggiungi: tag, le persone segnate in un post, classificazioni, fonti, traduzioni delle pagine di fumetti, voti con il momento in cui li hai dati, e ricerche salvate.
privacy-server-forum = Su un server pubblico, ciò che scrivi nel forum, con quando l'hai scritto e modificato l'ultima volta, e i tuoi voti sui tag. Un messaggio che elimini resta al suo posto, vuoto.
privacy-server-shares = Per ogni link di condivisione che crei: i post che contiene, il titolo, la scadenza, e quante volte e quando è stato aperto l'ultima volta (non da chi).
privacy-server-sites = Per gli altri siti: le ricerche che salvi su altri booru, gli account che salvi per loro (il nome utente come l'hai scritto, la chiave cifrata con il segreto del server), gli indirizzi che importi, tenuti 7 giorni dopo la fine, e ogni importazione di raccolte con i suoi conteggi finché non la rimuovi.
privacy-server-limits = Per rallentare i tentativi di indovinare le password, il server conta gli accessi falliti per account e per indirizzo di rete, e le registrazioni per indirizzo. Questi conteggi restano in memoria al massimo un'ora e non vengono mai scritti su disco.
privacy-server-guest-limits = Su un server pubblico, il server conta le richieste di chi sfoglia senza account in base al suo indirizzo di rete, perché nessuno chieda troppo in fretta. Questi conteggi restano in memoria per un minuto e non vengono mai scritti su disco.
privacy-device = Cosa resta sul tuo dispositivo
privacy-device-storage = Questa app tiene una sola voce nella memoria locale del browser: l'indirizzo del server, il tuo token di accesso, il tuo nome utente e ruolo, le tue preferenze (come il tema, la lingua, l'aspetto della galleria e i booru aggiunti per indirizzo), il punto in cui era la galleria nelle tue ultime 30 ricerche e gli ultimi 60 post che hai aperto.
privacy-device-sign-in = Mentre accedi tramite un provider di identità, la scheda tiene un codice monouso nella sua memoria di sessione, rimosso quando torni.
privacy-device-sign-out = Uscendo vengono rimossi il token, il nome utente e il ruolo, le ricerche ricordate dalla galleria e i post che hai aperto. L'indirizzo del server e le preferenze restano finché non cancelli i dati di questo sito nel browser.
privacy-device-cache = StarBooru non usa cookie. Immagini e video finiscono nella cache del browser, come su qualsiasi sito. L'app conserva i propri file (la pagina, il codice, gli stili e le icone) per potersi avviare anche senza rete; tra questi non c'è nessun tuo dato. Tutto il resto che carica resta nella memoria della scheda finché non la chiudi.
privacy-not-done = Cosa StarBooru non fa
privacy-not-done-tracking = Niente statistiche d'uso, pubblicità o tracciamento, e nessun servizio di terze parti oltre a un provider di identità o a un fornitore di IA che il gestore sceglie di configurare. Questa app carica codice, stili e icone solo da questo server.
privacy-not-done-contacts = Il server contatta altri siti solo quando cerchi, sfogli o importi da un altro booru, importi da un URL o accedi tramite il provider di identità del gestore, e quando un amministratore installa un modello di IA locale, che scarica da dove è pubblicato. Le sue richieste si presentano come StarBooru e non portano nessuno dei tuoi dati, tranne l'account che hai salvato per quel sito, che va solo a quel sito.
privacy-not-done-boorus = Quando sfogli un altro booru qui, il browser carica le immagini direttamente da quel sito, senza dire quale pagina le ha chieste, quindi il sito vede il tuo indirizzo di rete come lo vedrebbe sulle sue pagine. Le immagini dei siti che le mostrano solo sulle proprie pagine passano invece da questo server.
privacy-not-done-reverse = Quando cerchi un’immagine con Trova la fonte, il browser invia una copia ridotta al sito che scegli, e niente prima che tu ne scelga uno. iqdb e SauceNAO ricevono la copia caricata. Google Lens e trace.moe ricevono un link e la scaricano da questo server quando è raggiungibile da internet; altrimenti la incolli o la carichi tu lì. Quel sito vede il tuo indirizzo di rete, come sulle sue pagine.
privacy-not-done-ai = Se un amministratore configura un fornitore di IA, gli vengono inviate le miniature dei post per suggerire i tag (anche dei nuovi caricamenti, quando il tagging automatico è attivo), il testo letto in un’immagine quando lo traduci, e l’immagine stessa quando se ne traducono le vignette di fumetti o manga, a mano o per i nuovi caricamenti quando la traduzione automatica è attiva. Un fornitore sulla rete del gestore le tiene lì; i modelli locali girano sul server e niente ne esce.
privacy-not-done-library = Mentre si importa un’intera raccolta, la chiave che hai dato per quel sito resta sul server, e in ogni backup fatto nel frattempo; viene cancellata quando l’importazione finisce.
privacy-not-done-logs = Il contenuto dei file, il testo letto nelle immagini, le password e i token non vengono mai scritti nei log del server.
privacy-who-sees = Chi può vedere i tuoi post
privacy-who-sees-body = Chiunque abbia un account su questo server può vedere i post che contiene, e scaricarne i file originali con tutto ciò che contengono, come il luogo in cui è stata scattata una foto. Un post mostra il suo punteggio, non chi l'ha votato. I link ai file forniti all'app sono firmati e scadono, quindi un link copiato smette di funzionare entro 12 ore. Se crei un link di condivisione, chiunque lo abbia può vedere i post che contiene, senza i loro tag, finché non scade o non lo rimuovi.
privacy-who-sees-public = Su un server pubblico, anche chi non ha un account può vedere post, tag ed Esplora e scaricare i file, ma non il cestino né chi ha votato. Non può votare, caricare né salvare nulla. Lì un post si condivide con il suo indirizzo, che chiunque può aprire.
privacy-who-sees-profiles = Un server pubblico mostra a tutti, anche a chi non ha un account, il profilo di ogni account (immagine, nome visualizzato, presentazione, data di iscrizione e quanto ha pubblicato), chi ha caricato ogni post, e il forum con chi ha scritto cosa. I voti sui tag si vedono solo come punteggi. Rendere privato il server elimina le immagini, le presentazioni, il forum e i voti sui tag.
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
boorus-not-here = Niente altri booru qui
boorus-not-here-body = Questo server è pubblico, quindi non sfoglia altri booru.
boorus-not-here-back = Torna a Esplora
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
share-pick-day = Fino a una data
share-day = Ultimo giorno in cui il link funziona
share-day-missing = Scegli fino a quale giorno funziona il link.
share-day-soon = Scegli domani o un giorno successivo.
share-day-late = Scegli il { $last } o prima: al massimo { $max } giorni.
share-create = Crea link
share-ready = Link pronto. Chiunque lo abbia può aprirlo.
share-link = Link di condivisione
share-copy = Copia link
share-copied = Copiato
share-copy-failed = Non è stato possibile copiare il link. È selezionato: copialo dalla tastiera o dal menu.
share-address-title =
    { $count ->
        [one] Condividi questo post
       *[other] Condividi { $count } post
    }
share-address-lead =
    { $count ->
        [one] Questo server è pubblico: chiunque abbia questo indirizzo può vedere il post, senza un account.
       *[other] Questo server è pubblico: chiunque abbia questi indirizzi può vedere i post, senza un account.
    }
share-addresses = Indirizzi
share-copy-all = Copia tutti
share-copied-all = Copiati
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
# Un trattino morbido: sul telefono la parola può andare a capo tra le schede.
admin-tab-updates = Aggiorna{"\u00AD"}menti
admin-tab-moderation = Modera{"\u00AD"}zione
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
admin-profile = Profilo
admin-profile-named = Profilo di { $name }
admin-profile-lead = Ciò che questo account mostra a tutti. Ogni voce sparisce subito.
admin-clear-picture = Togli immagine
admin-clear-name = Cancella nome visualizzato
admin-clear-about = Cancella la presentazione
admin-cleared-picture = Immagine tolta.
admin-cleared-name = Nome visualizzato cancellato.
admin-cleared-about = Presentazione cancellata.
admin-moderation-private = Su un server privato non c'è nulla da moderare.
admin-voted-off = Tag tolti dai voti
admin-voted-off-lead = I tag che le persone hanno tolto dai post con i voti, dai più recenti. Rimetterne uno azzera i suoi voti, e nient'altro può farlo.
admin-voted-off-none = Nessun tag è stato tolto dai voti.
admin-voted-off-restore = Rimetti
admin-voted-off-back = { $tag } è di nuovo sul post n. { $id }.
admin-deleted = Account { $name } eliminato.
admin-server-lead = Si applicano subito e hanno la precedenza sulle variabili d'ambiente del server.
admin-settings-unread = Le impostazioni non sono ancora state caricate. Ricarica la pagina e riprova.
admin-home-title = Pagina iniziale
admin-home-on = Mostra una pagina iniziale
admin-home-on-hint = L'app si apre lì, e il logo ci riporta.
admin-home-heading = Titolo
admin-home-subtitle = Riga sotto il titolo
admin-home-query = Sfondo
admin-home-query-placeholder = I post più popolari classificati sicuri
admin-home-posts = Immagini
admin-home-preview = Come apparirà la pagina iniziale
admin-home-open = Apri la pagina iniziale
home-error-title = Il titolo supera i { $max } caratteri.
home-error-subtitle = La riga sotto il titolo supera i { $max } caratteri.
home-error-control = Il titolo o la riga sotto contiene un carattere non ammesso.
home-error-query-long = La ricerca supera i { $max } caratteri.
home-error-deleted = Lo sfondo non può mostrare il cestino.
home-error-count = Scegli da { $min } a { $max } immagini.
admin-blacklist-title = Blacklist predefinita
admin-blacklist-lead = Per i nuovi account e per gli ospiti. Gli account esistenti tengono la propria.
mode-title = Chi può vedere questo server
mode-private = Privato
mode-public = Pubblico
mode-private-means = Solo chi ha un account può vedere i post. I link di condivisione tornano a funzionare e si possono configurare altri booru. Il forum, le immagini del profilo e i voti sui tag spariscono.
mode-public-means = Chiunque può sfogliare post, tag, Esplora e il forum senza un account. I voti diventano Mi piace, un post si condivide con il suo indirizzo e ognuno ha un profilo.
mode-switch-to = Passa a { $mode }
mode-delete-and-switch = Elimina e cambia
mode-switch-without-backup = Cambia senza backup
mode-switching = Passaggio a { $mode }…
mode-now = Ora il server è { $mode }.
mode-done = Fatto
mode-nothing = Non viene eliminato né convertito nulla.
mode-deletes = Il cambio elimina definitivamente { $what }.
mode-backup-first = Prima fai un backup del database
mode-backup-hint = Consigliato. Il backup compare in Backup.
mode-backup-made = Il backup { $name } è in Backup
mode-step-backup = Backup del database
mode-step-switch = Passaggio a { $mode }
mode-keep-open = Tieni aperta questa pagina. Il backup di un database grande richiede un po’.
mode-backup-unavailable = Questo server non può fare backup perché manca pg_dump. Puoi comunque cambiare senza.
mode-backup-running = È in corso un altro backup. Riprova quando è finito.
mode-backup-failed = Il backup non è riuscito, quindi non è cambiato nulla. Riprova, o cambia senza backup.
mode-switch-running = È già in corso un altro cambio. Aspetta che finisca.
mode-same = Il server è già in questa modalità. Ora questa pagina lo mostra.
scores-title = Cos’è un voto
scores-likes = Mi piace
scores-stars = Stelle
scores-likes-means = Un voto è un Mi piace o un Non mi piace, e i post si ordinano per Mi piace meno Non mi piace.
scores-stars-means = Un voto va da 1 a 5 stelle, e i post si ordinano per una valutazione equa delle loro stelle.
scores-switch-to = Passa a { $kind }
scores-now = Ora i voti sono { $kind }.
scores-public-likes = Un server pubblico conta i Mi piace. Passa a privato per votare con le stelle.
change-share-links =
    { $count ->
        [one] 1 link di condivisione
       *[other] { $count } link di condivisione
    }
change-booru-accounts =
    { $count ->
        [one] 1 account booru salvato
       *[other] { $count } account booru salvati
    }
change-remote-searches =
    { $count ->
        [one] 1 ricerca booru salvata
       *[other] { $count } ricerche booru salvate
    }
change-star-votes =
    { $count ->
        [one] 1 voto a stelle
       *[other] { $count } voti a stelle
    }
change-three-stars =
    { $count ->
        [one] 1 voto da 3 stelle
       *[other] { $count } voti da 3 stelle
    }
change-like-votes =
    { $count ->
        [one] 1 Mi piace o Non mi piace
       *[other] { $count } Mi piace e Non mi piace
    }
change-profile-pictures =
    { $count ->
        [one] 1 immagine del profilo
       *[other] { $count } immagini del profilo
    }
change-profile-texts =
    { $count ->
        [one] 1 presentazione del profilo
       *[other] { $count } presentazioni del profilo
    }
change-forum-threads =
    { $count ->
        [one] 1 discussione del forum
       *[other] { $count } discussioni del forum
    }
change-forum-posts =
    { $count ->
        [one] 1 messaggio del forum
       *[other] { $count } messaggi del forum
    }
change-tag-votes =
    { $count ->
        [one] 1 voto su un tag
       *[other] { $count } voti sui tag
    }
change-tag-removals =
    { $count ->
        [one] 1 registro di un tag tolto dai voti
       *[other] { $count } registri di tag tolti dai voti
    }
change-fate-deleted = verranno eliminati
change-done-deleted = eliminati
change-fate-likes = diventeranno Mi piace e Non mi piace: 4 o 5 stelle un Mi piace, 1 o 2 un Non mi piace
change-done-likes = diventati Mi piace e Non mi piace
change-fate-neither = verranno eliminati, perché 3 stelle non è né un Mi piace né un Non mi piace
change-done-neither = eliminati, perché 3 stelle non è né un Mi piace né un Non mi piace
change-fate-stars = diventeranno stelle: un Mi piace 5, un Non mi piace 1
change-done-stars = diventati stelle
change-fate-texts = verranno eliminate; i nomi visualizzati restano
change-done-texts = eliminate; i nomi visualizzati restano
change-fate-threads = verranno eliminate; le categorie restano
change-done-threads = eliminate; le categorie restano
change-fate-removals = verranno eliminati; i tag restano tolti, e le modifiche possono rimetterli
change-done-removals = eliminati; i tag restano tolti, e le modifiche possono rimetterli
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
admin-tag-votes = I voti tolgono un tag a
admin-tag-votes-hint = Il punteggio a cui un tag viene tolto da un post. Con 0 non ne viene mai tolto nessuno.
admin-tag-votes-at = Un tag viene tolto da un post quando il suo punteggio arriva a { $score }.
admin-tag-votes-off = I voti non tolgono mai un tag.
admin-tag-votes-error = Inserisci un numero da 0 a { $max }.
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

## Aggiornamenti: l'avviso in cima alle pagine, e Amministrazione › Aggiornamenti

updates-notice = È uscito StarBooru { $version }
updates-notice-more = Scopri le novità e aggiorna
updates-notice-label = Nuova versione
updates-notice-hide = Nascondi fino alla prossima versione
updates-notice-hidden = Nascosto finché non esce una versione più recente.
updates-overview-new = È uscita la { $version }
updates-old-server = Questo server non dice se è uscito uno StarBooru più recente: è precedente alla 0.2.0. Aggiornalo a mano, dove si trova il suo file compose:
updates-title = Versione
updates-running = In uso
updates-newest = Più recente
updates-up-to-date = Aggiornato
updates-released = Uscita { $when }
updates-checked = Controllato { $when }
updates-not-checked = Non ancora controllato
updates-check-failed = L'ultimo controllo non è riuscito: { $reason }
updates-checks-off = I controlli delle nuove versioni sono disattivati.
updates-check-now = Controlla ora
updates-checking = Controllo…
updates-check-wait = Di nuovo tra { $seconds } s
updates-checked-new = È uscito StarBooru { $version }.
updates-checked-none = Niente di più recente: la { $version } è l'ultima.
updates-backup-first = Prima fai un backup del database
updates-install = Aggiorna alla { $version }
updates-confirm = Il server si riavvia per passare alla { $version }: resta irraggiungibile per qualche istante, poi questa pagina si ricollega e si ricarica da sola.
updates-confirm-no-backup = Prima non verrà fatto alcun backup.
updates-confirm-go = Aggiorna ora
updates-cancel = Annulla
updates-updating = Aggiornamento alla { $version }
updates-automatic = Automatico
updates-step-checking = Preparazione
updates-step-checking-note = Verifica che la { $version } si possa installare qui.
updates-step-downloading = Download
updates-step-downloading-note = Scaricamento di StarBooru { $version }.
updates-step-backing-up = Backup
updates-step-backing-up-note = Prima viene salvata una copia del database.
updates-step-restarting = Riavvio
updates-step-restarting-note = Passaggio alla nuova versione. Il server resta irraggiungibile per qualche istante; questa pagina lo aspetta.
updates-step-done = Aggiornato
updates-step-done-note = StarBooru { $version } è in funzione.
updates-step-failed = Non aggiornato
updates-step-failed-note = L'aggiornamento si è fermato, e la versione di prima è ancora in funzione.
updates-step-done-mark = fatto
updates-step-skipped = Saltato
updates-slow = Ci sta mettendo più del solito. Il log del server dice come sta andando.
updates-done = Aggiornato alla { $version }
updates-reloading = Ricaricamento…
updates-result-done = Aggiornato dalla { $from } alla { $to } il { $time }.
updates-result-done-auto = Aggiornato automaticamente dalla { $from } alla { $to } il { $time }.
updates-result-failed = L'aggiornamento alla { $to } del { $time } non è riuscito. { $reason } StarBooru { $current } è ancora in funzione.
updates-result-failed-auto = L'aggiornamento automatico alla { $to } del { $time } non è riuscito. { $reason } StarBooru { $current } è ancora in funzione, e il prossimo tentativo è domani.
updates-result-no-reason = Il server non ha detto perché.
updates-backup-made = Backup fatto prima: { $name }
updates-cannot = Questo server non può aggiornarsi da solo.
updates-why-no-docker-socket = Gira in un container Docker che non raggiunge il socket di Docker, quindi non può sostituirsi.
updates-why-docker-unreachable = Raggiunge il socket di Docker, ma Docker ha rifiutato o non conosceva questo container.
updates-why-unknown-image = La sua immagine non è né latest né una versione, come una creata su questa macchina, quindi non ce n'è una più recente da scaricare.
updates-why-removed-on-stop = Il suo container viene rimosso quando si ferma (docker run --rm), quindi non potrebbe tornare se l'aggiornamento non riuscisse.
updates-why-development-build = È stato compilato dai sorgenti, non installato da una release.
updates-why-not-supervised = Niente, come systemd, lo riavvierebbe dopo l'aggiornamento.
updates-why-read-only-folder = Non può scrivere nella propria cartella.
updates-why-unsupported = Non esiste un pacchetto del server per questo sistema.
updates-why-unknown = Non ha detto perché.
updates-how-compose = Aggiornalo a mano, dove si trova il suo file compose:
updates-how-redeploy = Su Dokploy, premi invece Redeploy.
updates-how-socket = Per aggiornarlo da qui, dai al container il socket di Docker, come mostra la guida al deployment.
updates-how-swarm = Rifai il deploy del servizio con la nuova immagine.
updates-how-image = Crea o scarica l'immagine della nuova versione, poi crea di nuovo il container.
updates-how-removed-on-stop = Scarica la nuova immagine e avvia di nuovo il container. Avviato senza --rm, può aggiornarsi da qui.
updates-how-source = Aggiorna i sorgenti e compilalo di nuovo.
updates-how-systemd = Installa a mano il nuovo pacchetto. Avviato da systemd, può aggiornarsi da qui.
updates-how-folder = Installa a mano il nuovo pacchetto, oppure lascia che il server scriva nella sua cartella per aggiornarlo da qui.
updates-how-manual = Installa a mano la nuova versione.
updates-copy = Copia
updates-copied = Copiato
updates-copy-failed = Il browser non l'ha copiato: seleziona il comando e copialo da tastiera o dal menu.
updates-pinned-before = Il file compose indica la versione di questo server (STARBOORU_VERSION). Quando aggiorni, cambiala anche lì in { $version }, altrimenti il prossimo redeploy torna alla versione vecchia.
updates-pinned-after = Il file compose indica una versione (STARBOORU_VERSION). Controlla che sia { $version }, altrimenti il prossimo redeploy torna a una più vecchia.
updates-notes-new = Novità della { $version }
updates-notes = Note di rilascio della { $version }
updates-notes-empty = Questa release non ha note.
updates-release-page = Apri la release su GitHub
updates-settings-title = Controlli e aggiornamenti automatici
updates-check-setting = Cerca nuove versioni
updates-check-setting-hint = Il server chiede a GitHub ogni 6 ore, senza dirgli nulla di sé.
updates-auto-setting = Aggiorna automaticamente
updates-auto-setting-hint = Prima con un backup, all'ora qui sotto.
updates-auto-needs-check = Servono i controlli delle nuove versioni.
updates-auto-unavailable = Questo server non ha aggiornamenti automatici.
updates-hour = Ogni giorno alle
updates-hour-label = Ora degli aggiornamenti automatici
updates-hour-zone = sull'orologio del server ({ $zone })
updates-hour-here = qui le { $time }
updates-saved = Salvato
updates-error-up-to-date = Questo server ha già l'ultima versione.
updates-error-cannot = Questo server non può aggiornarsi da solo.
updates-error-no-backup = Questo server non può fare backup, quindi non può farne uno prima. Togli la spunta da "Prima fai un backup del database" per aggiornare senza.
updates-error-running = C'è già un aggiornamento in corso.
updates-error-checks-off = I controlli delle nuove versioni sono disattivati. Prima attivali qui sotto.

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
