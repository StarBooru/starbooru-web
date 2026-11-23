# Testi dell'interfaccia di StarBooru, italiano.

app-name = StarBooru

## Shared

notfound-title = Qui non c'è niente
notfound-body = Non esiste una pagina { $path }.
notfound-back = Torna alla galleria
notice-close = Chiudi
soon-body = Questa pagina è ancora in costruzione. Torna a trovarci presto.

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

## Page titles

post-number = Post #{ $id }
trash-title = Cestino
dup-title = Trova duplicati
help-title = Guida alla ricerca
admin-title = Amministrazione
shared-title = Post condivisi
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
post-download = Scarica
download-failed = Download non riuscito: { $reason }
share-button = Condividi
editor-open = Modifica immagine o video
editor-image-short = Modifica immagine
editor-video-short = Taglia
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
