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
settings-library = Raccolta
settings-trash-note = Ripristina o elimina definitivamente i post

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

explore-window = Tendenze di
explore-today = Oggi
explore-week = Settimana
explore-month = Mese
explore-year = Anno
explore-phrase-today = oggi
explore-phrase-week = questa settimana
explore-phrase-month = questo mese
explore-phrase-year = quest'anno
explore-phrase-days = negli ultimi { $count } giorni
explore-nothing = Ancora niente da esplorare
explore-nothing-body = Carica immagini o video e i migliori compariranno qui.
explore-everything = Esplora tutto
nothing-matches = Nessun risultato per “{ $query }”.
explore-trending = Di tendenza
explore-no-upvotes = Nessun voto positivo { $when }. Vota i post che ti piacciono e compariranno qui.
explore-popular-tags = Tag popolari
explore-tags-note = sui post aggiunti { $when }
explore-no-tags = Nessun post con tag aggiunto { $when }.
explore-added = Post aggiunti { $when }
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
upload-progress = { $done } di { $count } completati
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
