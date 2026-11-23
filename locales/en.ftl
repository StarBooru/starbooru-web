# StarBooru UI text, English (the reference catalogue).

app-name = StarBooru

## Shared

notfound-title = Nothing here
notfound-body = There is no page at { $path }.
notfound-back = Back to the gallery
notice-close = Close
soon-body = This page is still being built. Check back soon.

## Navigation

nav-main = Main
nav-gallery = Gallery
nav-explore = Explore
nav-boorus = Boorus
nav-tags = Tags
nav-upload = Upload
nav-settings = Settings
nav-settings-tab = Settings
search-placeholder = Search tags, e.g. cat rating:safe
search-label = Search tags
search-clear = Clear the search
search-suggestions = Tag suggestions
search-hint = { $tag }, { $count ->
    [one] 1 post
   *[other] { $count } posts
}
search-corrected = Spelling corrected: { $tags }
search-exact = Search exactly as typed

## The empty search box's demo

demo-tag = Posts with this tag
demo-tag-all = Posts with every tag listed
demo-tag-not = A minus leaves a tag out
demo-figure = One person has all of these
demo-rating = Only this rating
demo-rating-any = Any of these ratings
demo-rating-not = Any rating but explicit
demo-type = Only videos
demo-date = From March 2024
demo-date-range = From 2023 to the end of 2024
demo-date-age = Less than 7 days old
demo-size = Wider than 1920 pixels
demo-ratio = Width over height is 16:9
demo-color = Has this colour
demo-text = This word is written in the picture
demo-order-score = Highest score first
demo-order-random = A new shuffle every time
demo-combined = Put them all together

## Page titles

post-number = Post #{ $id }
trash-title = Recycle bin
dup-title = Duplicate finder
help-title = Search cheat sheet
admin-title = Administration
shared-title = Shared posts
privacy-title = Privacy Policy
terms-title = Terms of Service

## Errors

error-session-expired = Your session has expired. Sign in again.
error-unreachable = Can't reach the server at { $server }. Check the address and your connection, then try again.
error-bad-address = That isn't a server address. Enter one like https://booru.example.com.
error-not-a-server = { $server } answers, but it isn't a StarBooru server. Check the address.
error-rate-limited = Too many requests. Try again later.
error-rate-limited-minutes =
    { $minutes ->
        [one] Too many attempts. Try again in a minute.
       *[other] Too many attempts. Try again in { $minutes } minutes.
    }

## Sign-in

login-tagline = Your private image library
login-server-version = { $name } · v{ $version }
login-first-user = This server has no accounts yet. The account you create now becomes the administrator.
login-server = Server
login-server-placeholder = https://booru.example.com
login-username = Username
login-password = Password
login-wait = Please wait…
login-create-admin = Create administrator account
login-create = Create account
login-sign-in = Sign in
login-have-account = I already have an account
login-want-account = Create an account
login-wrong = Wrong username or password.
login-weak-password = This password is easy to guess. Try a longer one, or a few words that don't go together.
login-or = or
login-with = Sign in with { $provider }
password-strength-weak = Easy to guess
password-strength-fair = Could be stronger
password-strength-strong = Strong password
oidc-finishing = Signing you in…
oidc-denied = The sign-in was cancelled.
oidc-failed = The sign-in didn't work. Try again, or ask the administrator.
oidc-expired = This sign-in has expired. Start again.
legal-nav = Legal
legal-privacy-short = Privacy
legal-terms-short = Terms

## Settings

settings-account = Account
settings-signed-in-as = Signed in as
settings-server-version = Server version
settings-sign-out = Sign out

## Gallery

retry = Retry
loading = Loading
load-more = Load more
no-matches = No matches
post-alt = Post { $id }
badge-gif = GIF
gallery-tag-info = About this tag
gallery-syntax = Search syntax
gallery-to-top = Back to the top
gallery-empty = No posts yet
gallery-empty-body = Upload images or videos to start your collection.
gallery-try-fewer = Nothing matches this search. Try fewer tags.
sort = Sort
sort-by = Sort by
sort-new = Newest
sort-old = Oldest
sort-score = Top score
sort-updated = Recently edited
sort-size = Largest file
sort-size-asc = Smallest file
sort-pixels = Highest resolution
sort-longside = Longest side
sort-tags = Most tags
sort-tags-asc = Fewest tags
sort-random = Random
view-title = Gallery layout
view-shape = Tile shape
view-square = Square
view-landscape = Landscape
view-portrait = Portrait
view-staggered = Staggered
view-staggered-by = Staggered shapes
view-everything = Everything
view-columns = { $count ->
    [one] 1 column
   *[other] { $count } columns
}
view-per-row = About { $count } per row
view-columns-auto = Automatic
view-flush = Remove the spaces and rounded corners between images
color-search = Search by colour
color-wheel = Pick a colour from the wheel
color-hex = Hex colour
color-hex-apply = Use this colour
color-hex-invalid = Use 3 or 6 hex digits, like #2060e0
color-clear = Any colour
color-red = Red
color-orange = Orange
color-yellow = Yellow
color-green = Green
color-teal = Teal
color-blue = Blue
color-purple = Purple
color-pink = Pink
color-brown = Brown
color-black = Black
color-gray = Gray
color-white = White

## Viewer

post-missing-title = No post { $id }
post-missing-body = It may have been deleted for good, or the address is wrong.
post-back = Back
post-prev-title = Previous (←)
post-prev = Previous post
post-next-title = Next (→)
post-next = Next post
zoom-out-title = Zoom out (−)
zoom-out = Zoom out
zoom-fit = Fit
zoom-fit-title = Fit (0)
zoom-reset = Reset zoom
zoom-in-title = Zoom in (+)
zoom-in = Zoom in
fullscreen-exit-title = Exit fullscreen (F)
fullscreen-exit = Exit fullscreen
fullscreen-title = Fullscreen (F)
fullscreen = Fullscreen
post-actions = Post actions
post-upvote = Upvote
post-downvote = Downvote
post-download = Download
download-failed = Download failed: { $reason }
share-button = Share
editor-open = Edit image or video
editor-image-short = Edit image
editor-video-short = Trim
post-delete = Delete
post-delete-title = Move to recycle bin
press-deleted = Moved to the recycle bin
press-undo = Undo
post-in-trash = This post is in the recycle bin.
trash-restore = Restore
post-tags = Tags
post-edit = Edit
post-edit-tags = Edit tags
post-no-tags = No tags yet.
category-general = General
category-artist = Artist
category-character = Character
category-copyright = Copyright
category-meta = Meta
section-people = People
section-hair = Hair
section-eyes = Eyes
section-skin = Skin
section-body = Body
section-expression = Expression
section-pose = Pose and framing
section-clothing = Clothing
section-setting = Setting
section-style = Style
section-other = Other
post-details = Details
rating = Rating
rating-safe = safe
rating-sensitive = sensitive
rating-questionable = questionable
rating-explicit = explicit
post-size = Size
post-dimensions = Dimensions
post-length = Length
post-type = Type
post-file-date = File date
post-colors = Colours
post-color-search = Search for { $color }
post-added = Added
post-source = Source
post-sample-note = Showing a smaller copy. Zooming in loads the original.
post-show-original = Show original

## Search cheat sheet

help-try = Try a query
help-try-label = Query to explain
help-try-placeholder = cat -dog rating:safe order:score
help-search = Search
help-search-for = Search for { $term }
help-tags = Tags
help-tag = has the tag
help-tag-all = has every tag listed
help-tag-not = doesn't have the tag
help-tag-tree = has footwear or any tag under it, like boots or sneakers
help-figure = one person in the picture has all of these tags; girl, boy or other says who. A minus in front finds posts where no one does
help-tag-category = the tag some_name; when tagging, the prefix makes it an artist tag (also character:, copyright: and meta:)
help-tag-spaces = two tags: spaces separate terms, so write blue_sky
help-tag-meta = high-resolution posts without sound. The server adds meta tags like lowres, highres, absurdres, wide_image, tall_image, hd, full_hd, 4k, 8k, horizontal_image, vertical_image, square_image, animated, video, sound and long_video by itself
help-filters = Filters
help-rating = rated safe, sensitive (s), questionable (q) or explicit (e)
help-rating-any = any of the listed ratings
help-rating-not = any rating but explicit
help-type = still images; also type:animation and type:video
help-deleted = posts in the recycle bin
help-date = the date in the file, or the upload date if it has none; also date:2024 and date:2024-03-15
help-date-range = from the start of the first period to the end of the last; date:2023.. and date:..2024 leave one end open
help-date-compare = after that period; also >=, < and <=
help-date-age = more than 30 days old; date:<7d is less than 7 days old. Also h, w, mo and y
help-size = wider than 1920 pixels; also height:, >=, <, <=, an exact width:1920 and a range width:1000..2000
help-mpixels = more than 8 megapixels; a bare mpixels:8 is about 8
help-ratio = width over height, as 16:9 or 1.78, within 0.01; ratio:1 is square and ratio:>1 is wider than tall
help-size-not = any size but these; a post of unknown size stays in
help-color = has this colour or, when no post does, the nearest one stored; also orange, yellow, green, teal, blue, purple, pink, brown, black, gray and white
help-color-hex = any colour, in hex; also #2060e0 and 26e
help-color-not = doesn't have this colour
help-exact = an unknown tag is normally corrected to the closest one in use; this searches tags exactly as typed
help-text = Text in images
help-text-word = the text read in the image has the word (any case)
help-text-phrase = has all these words; quotes keep them together
help-text-not = the text read in the image doesn't have the word
help-sorting = Sorting
help-order-new = newest first (the default); order:old for oldest
help-order-score = highest score first
help-order-size = largest file first
help-order-pixels = highest resolution first; order:mpixels too
help-order-longside = longest side first
help-order-tags = most tags first
help-order-updated = most recently edited first
help-order-direction = add _asc or _desc to set the direction
help-order-random = a new shuffle every time
help-combined = Combined
help-combined-cats = safe cats without dogs, best first
help-combined-videos = the biggest videos
help-tips = Good to know
help-tip-normalise = Tags are lower-cased, and spaces or repeated underscores become one _.
help-tip-aliases = An alias finds its tag, and implied tags are added when you tag a post.
help-tip-terms = A query holds at most 40 terms.
help-shortcuts = Keyboard shortcuts and gestures
key-prev-next = previous or next post
key-fullscreen = full screen
key-zoom = zoom in or out
key-fit = fit to the screen
key-escape = leave full screen, then back to the gallery
key-double-click = Double click
key-double-click-action = zoom in on a point, again to fit
key-swipe = Swipe left or right
key-swipe-action = previous or next post
people-girl = Girl
people-boy = Boy
people-other = Other
explain-deleted = In the recycle bin
explain-tagged = Tagged { $tag }
explain-tagged-all = Tagged with all of { $tags }
explain-not-tagged = Not tagged { $tag }
explain-tagged-none = Tagged with none of { $tags }
explain-rated = Rated { $ratings }
explain-not-rated = Not rated { $ratings }
list-or = { $rest } or { $last }
explain-images = Still images only
explain-animations = Animations only
explain-videos = Videos only
explain-figure = One person has all of { $tags }
explain-no-figure = No one has all of { $tags }
explain-figure-kind = A person ({ $kind }) has all of { $tags }
explain-no-figure-kind = No person ({ $kind }) has all of { $tags }
explain-figure-only = A person marked as { $kind }
explain-no-figure-only = No person marked as { $kind }
explain-date = Dated { $value }
explain-not-date = Not dated { $value }
explain-size = { $field } { $value }
explain-not-size = { $field } not { $value }
explain-color = Has the colour { $value }, or the nearest one stored
explain-not-color = Doesn't have the colour { $value }
explain-exact = Tags are taken exactly as typed
explain-size-width = Width
explain-size-height = Height
explain-size-mpixels = Megapixels
explain-size-ratio = Aspect ratio
explain-text = Text in the image has “{ $text }”
explain-not-text = Text in the image doesn’t have “{ $text }”
explain-every = Every post
explain-order-new = Newest first
explain-order-old = Oldest first
explain-order-random = In random order
explain-order-score-desc = By score, highest first
explain-order-score-asc = By score, lowest first
explain-order-size-desc = By file size, largest first
explain-order-size-asc = By file size, smallest first
explain-order-pixels-desc = By resolution, highest first
explain-order-pixels-asc = By resolution, lowest first
explain-order-longside-desc = By longest side, longest first
explain-order-longside-asc = By longest side, shortest first
explain-order-tags-desc = By tag count, most first
explain-order-tags-asc = By tag count, fewest first
explain-order-updated-desc = By last edit, latest first
explain-order-updated-asc = By last edit, earliest first
