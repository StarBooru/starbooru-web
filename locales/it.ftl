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
