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
settings-library = Library
settings-trash-note = Restore or permanently delete posts
settings-dup-note = Find posts that look alike and keep the best copy

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

search-save = Save this search
search-forget = Forget this search
search-save-failed = Couldn't save the search: { $reason }
search-forget-failed = Couldn't forget the search: { $reason }
saved-searches = Saved searches
saved-remove = Remove “{ $name }”
saved-removed = Removed “{ $name }”

gallery-select = Select posts
gallery-stop-selecting = Stop selecting
bulk-label = Edit selected posts
bulk-selected =
    { $count ->
        [one] 1 selected
       *[other] { $count } selected
    }
bulk-select-all = Select all
bulk-select-none = Select none
bulk-edit-title = Edit tags and rating
bulk-sheet-title =
    { $count ->
        [one] Edit 1 post
       *[other] Edit { $count } posts
    }
bulk-add-tags = Add these tags
bulk-remove-tags = Remove these tags
bulk-keep-rating = Keep rating
bulk-apply = Apply
bulk-tag-both = “{ $tag }” is in both lists. Keep it in one.
bulk-nothing = Add or remove a tag, or pick a rating.
bulk-delete-failed = Couldn't move them to the recycle bin: { $reason }
bulk-restore-failed =
    { $count ->
        [one] Couldn't bring 1 post back: { $reason }
       *[other] Couldn't bring { $count } posts back: { $reason }
    }
bulk-updated =
    { $count ->
        [one] Updated 1 post
       *[other] Updated { $count } posts
    }
bulk-moved =
    { $count ->
        [one] Moved 1 post to the recycle bin
       *[other] Moved { $count } posts to the recycle bin
    }
bulk-skipped =
    { $count ->
        [one] skipped 1 post
       *[other] skipped { $count } posts
    }
bulk-skipped-undeletable =
    { $count ->
        [one] skipped 1 post you can't delete
       *[other] skipped { $count } posts you can't delete
    }
bulk-done = { $done }.
bulk-done-skipped = { $done }; { $skipped }.

## Explore

explore-window = Trending over
explore-today = Today
explore-week = Week
explore-month = Month
explore-year = Year
explore-phrase-today = today
explore-phrase-week = this week
explore-phrase-month = this month
explore-phrase-year = this year
explore-phrase-days = in the last { $count } days
explore-nothing = Nothing to explore yet
explore-nothing-body = Upload images or videos, and the best of them show up here.
explore-everything = Explore everything
nothing-matches = Nothing matches “{ $query }”.
explore-trending = Trending
explore-no-upvotes = No upvotes { $when }. Vote on posts you like and they show up here.
explore-popular-tags = Popular tags
explore-tags-note = on posts added { $when }
explore-no-tags = No tagged posts were added { $when }.
explore-added = Posts added { $when }
explore-on-this-day = On this day
explore-no-memories = Nothing was added on this date in earlier years.
explore-years-ago = { $count ->
    [one] 1 year ago
   *[other] { $count } years ago
}
explore-random = Random picks
explore-shuffle = Shuffle
explore-filter-placeholder = Narrow down, e.g. -rating:e
explore-filter-label = Narrow down with a search

## Recycle bin

trash-lead = Deleted posts wait here until you restore them or delete them for good. Deleting for good also removes the file from the server.
trash-purge-after =
    { $days ->
        [one] The server deletes posts here for good a day after they were moved here.
       *[other] The server deletes posts here for good { $days } days after they were moved here.
    }
trash-purge-never = Posts stay here until someone deletes them for good.
trash-none = The recycle bin is empty
trash-restore-post = Restore post { $id }
trash-restore-all = Restore all
trash-restore-all-confirm = Restore everything?
trash-restored =
    { $count ->
        [one] Restored 1 post.
       *[other] Restored { $count } posts.
    }
trash-restore-failed = Couldn't restore post { $id }: { $reason }
trash-purge = Delete for good
trash-purge-post = Delete post { $id } for good
trash-purge-failed = Couldn't delete post { $id } for good: { $reason }
trash-purged =
    { $count ->
        [one] Deleted 1 post for good.
       *[other] Deleted { $count } posts for good.
    }
trash-empty = Empty recycle bin
trash-empty-confirm = Delete everything for good?
sure = Sure?

## Duplicates

dup-matching = Matching
dup-strict = Strict
dup-normal = Normal
dup-loose = Loose
dup-lead = Posts that look alike, closest matches first. Keep the best copy and the others go to the recycle bin.
dup-unhashed =
    { $count ->
        [one] 1 post can't be compared yet: it has no thumbnail, or is still being fingerprinted.
       *[other] { $count } posts can't be compared yet: they have no thumbnail, or are still being fingerprinted.
    }
dup-none = No duplicates found
dup-none-try-loose = Try Loose matching to catch edited copies.
dup-none-loose = Nothing here looks alike.
dup-groups =
    { $count ->
        [one] 1 group
       *[other] { $count } groups
    }
posts-count =
    { $count ->
        [one] 1 post
       *[other] { $count } posts
    }
dup-some-of = { $shown } of { $count } posts
dup-keep-which = Post to keep
dup-merge = Copy their tags to the kept post
dup-dismiss = Not duplicates
dup-resolve =
    { $count ->
        [one] Keep one, bin the other
       *[other] Keep one, bin { $count } others
    }
dup-unknown-size = Unknown size
dup-keep = Keep
dup-identical = Look identical
dup-nearly-identical = Nearly identical
dup-very-similar = Very similar
dup-similar = Similar
tags-count =
    { $count ->
        [one] 1 tag
       *[other] { $count } tags
    }
dup-added = Added { $date }
dup-post = #{ $id }
dup-unmerge-failed = Couldn't take the copied tags off post #{ $id }: { $reason }

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
post-votes = Votes
post-score = Score
vote-failed = Couldn't save your vote: { $reason }
post-download = Download
download-failed = Download failed: { $reason }
share-button = Share
editor-open = Edit image or video
editor-image-short = Edit image
editor-video-short = Trim
editor-title-image = Edit image
editor-title-video = Trim video
editor-tools = Tools
editor-crop = Crop
editor-trim = Trim
editor-history = History
editor-undo = Undo
editor-redo = Redo
editor-undo-keys = Undo (Ctrl+Z)
editor-redo-keys = Redo (Ctrl+Shift+Z)
editor-reset = Start over
editor-save = Save
editor-save-copy = Save copy
editor-save-keys = Save as a new post (Ctrl+S)
editor-save-options = What happens to the original
editor-save-replace = Replace original
editor-save-replace-hint = The original goes to the recycle bin
editor-keep-original = Keep original
editor-keep-original-hint = The edit is saved as a new post beside it
editor-saved = Saved as a new post
editor-saved-binned = Original moved to the recycle bin
editor-undo-save = Undo
editor-ratio = Proportions
editor-ratio-free = Free
editor-ratio-original = Original
editor-ratio-custom = Custom
editor-ratio-swap = Portrait or landscape
editor-ratio-width = Width
editor-ratio-height = Height
editor-straighten = Straighten
editor-straighten-reset = Back to 0°
editor-turn-flip = Turn and flip
editor-turn-left = Turn left
editor-turn-right = Turn right
editor-flip-h = Flip left to right
editor-flip-v = Flip top to bottom
editor-crop-area = Crop. Arrow keys move it, and further with Shift.
editor-handle-n = Top edge
editor-handle-s = Bottom edge
editor-handle-w = Left edge
editor-handle-e = Right edge
editor-handle-nw = Top left corner
editor-handle-ne = Top right corner
editor-handle-sw = Bottom left corner
editor-handle-se = Bottom right corner
editor-timeline = Part to keep
editor-start = Start (seconds)
editor-end = End (seconds)
editor-now = Use current time
editor-now-title = Take the time the video is at now
editor-keeps = Keeps { $length } of { $total }.
editor-keyframe-hint = The original quality is kept where the video allows it; otherwise the clip is re-encoded, which can take a while.
editor-bad-time = Enter a start before the end, within the video.
editor-too-short = The clip must be at least a tenth of a second long.
editor-no-change = Make a change first.
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
post-edit-title = Edit tags and rating (E)
post-tags-hint = Prefix a tag with artist:, character:, copyright: or meta: to set its category.
post-did-you-mean = Did you mean?
post-replace-tag = Replace { $from } with { $to }
tag-offers = Matching tags
tag-error-prefix = “{ $tag }” can't be a tag: a tag can't start with - or ~.
tag-error-long = “{ $tag }” is too long for a tag. Keep it to { $max } characters.
tag-error-many = A post can have at most { $max } tags.
tag-error-other = “{ $tag }” can't be a tag.
cancel = Cancel
save = Save
ai-suggest = Suggest tags
ai-suggesting = Asking the AI…
ai-add-all = Add all
ai-new-tag = new
ai-no-suggestions = No suggestions for this post.
ai-rating = Suggested rating: { $rating }
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
people-title = People
people-none = No one is marked yet. Adding the people in the picture tags their clothes and looks one by one, and keeps 1girl, 2boys and solo right.
people-hint = Their tags go on the post too. Counts like 1girl and solo follow the people. Separate tags with spaces.
people-add = Add a person
people-edit = Edit
people-edit-label = Edit the people
people-remove = Remove this person
people-kind = Who
people-numbered = { $kind } { $n }
people-character = Character
people-cosplay = Cosplaying
people-as = as
people-summary-character = { $person }: { $character }
people-summary-cosplay = { $person }, cosplaying { $cosplay }
people-summary-both = { $person }: { $character } as { $cosplay }
people-clothing = Clothing
people-looks = Looks
people-error-many = A post can have at most { $max } people.
people-error-many-tags = { $person } has too many tags. One person can have at most { $max }.
slot-head = Head
slot-body = Body
slot-bottom = Bottom and legs
slot-shoes = Shoes
slot-wear = Other clothing
slot-shape = Build
slot-eyes = Eyes
slot-skin = Skin
slot-hair = Hair
slot-pose = Pose
slot-other = Other
ai-suggest-people = Suggest people
ai-no-people = The AI found no people.
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
post-related = Related
press-open = Open
press-open-tab = Open in a new tab
swipe-like = Like
press-unlike = Remove like
press-liked = Liked
press-unliked = Like removed
delete-failed = Couldn't move it to the recycle bin: { $reason }
swipe-select = Select

## Tags

tags-sort = Sort tags
tags-most-used = Most used
tags-a-z = A–Z
tags-filter-label = Filter tags
tags-filter-placeholder = Filter by name or alias
tags-any-category = All
tags-none-yet = No tags yet
tags-none-match = No tags match
tags-appear = Tags appear here once posts use them.
tags-missing = No tag called “{ $name }”
tags-all = All tags
tags-browse =
    { $count ->
        [one] Browse 1 post
       *[other] Browse { $count } posts
    }
tags-is-alias = “{ $name }” is an alias of this tag.
tags-category = Category
tags-tree = Place in the tree
tags-tree-hint = Searching a tag also finds the tags under it, and the top of the tree is the section a post's tags are grouped under.
tags-tree-top = At the top: nothing above it.
tags-parent = Parent tag
tags-move = Move under
tags-move-top = Move to the top
tags-children = Under it
tags-children-none = Nothing under it yet.
tags-aliases = Aliases
tags-aliases-hint = Other names for this tag. Tagging or searching with an alias uses this tag. Adding an alias that is already a tag merges it into this one.
tags-aliases-none = No aliases.
tags-implies = Implies
tags-implies-hint = Posts with this tag get these tags too, including posts tagged earlier.
tags-implies-none = Implies no other tags.
tags-implied-by = Implied by
tags-implied-by-hint = Tags that add this one.
tags-implied-by-none = No tag implies this one.
tags-new-alias = New alias
tags-implied-tag = Implied tag
tags-remove = Remove
tags-remove-named = Remove { $name }
tags-admins-only = Only admins can change this.
tags-merge-confirm =
    { $count ->
        [one] “{ $name }” is a tag of its own, on 1 post. As an alias it merges into this tag, and that can't be undone.
       *[other] “{ $name }” is a tag of its own, on { $count } posts. As an alias it merges into this tag, and that can't be undone.
    }
tags-merge = Merge
add = Add
tag-menu-search = Search this tag
tag-menu-add = Add to this search
tag-menu-exclude = Leave it out
tag-menu-details = Tag details
tag-menu-copy = Copy name
tag-copied = Tag name copied
tag-copy-failed = Couldn't copy the tag name

## Upload

upload-files = Choose files
upload-folder = Choose a folder
upload-folder-tags = Tag files with their folder name
upload-autotag = Let the AI tag these uploads
upload-link = Add a link
upload-clear = Clear list
upload-drop = Drop or paste files or links
upload-drop-now = Drop to upload
upload-urls-placeholder = Paste links to files or booru posts, one per line
upload-urls-label = Links to files or booru posts
upload-urls-only = Enter addresses that start with http:// or https://.
upload-skipped =
    { $more ->
        [0] Only images and videos are taken. Left out: { $names }.
       *[other] Only images and videos are taken. Left out: { $names } and { $more } more.
    }
upload-paste-nothing = There was no file or link to paste.
upload-shared-tags = Tags for all files
upload-source = Source
upload-source-placeholder = Source (optional)
upload-source-hint = Where the files come from. A link is its own source.
upload-start =
    { $count ->
        [0] Upload
        [one] Upload 1 file
       *[other] Upload { $count } files
    }
upload-progress = { $done } of { $count } done
upload-queued = Queued
upload-uploading = Uploading
upload-uploaded = Uploaded
upload-stopped = Cancelled
upload-failed = Failed
upload-known = Already here
upload-known-post = Already here as post #{ $id }
upload-open-post = Open post #{ $id }
upload-too-big = Bigger than this server takes ({ $max } at most).
upload-unsupported = This server only takes an image or a video in a format it knows.
upload-unreadable = Can't read the file: { $error }
upload-gone = The file is no longer at hand. Add it again.
upload-file-tags-placeholder = Tags for this file only
upload-file-tags-label = Tags for { $name }
upload-remove = Remove { $name }
upload-cancel = Stop uploading { $name }
upload-retry = Try { $name } again
upload-open = Open { $name }
upload-close = Close
upload-remove-item = Remove from upload
upload-open-link = Open link
upload-rating-usual = Same as the rest ({ $rating })
upload-done = Imported
upload-job-running = Importing
upload-job-cancel = Cancel
activity-pending =
    { $count ->
        [one] 1 upload or import in progress
       *[other] { $count } uploads and imports in progress
    }
activity-uploaded =
    { $count ->
        [one] 1 file uploaded
       *[other] { $count } files uploaded
    }
activity-imported =
    { $count ->
        [one] 1 post imported
       *[other] { $count } posts imported
    }
activity-known = { $count } already here
activity-failed = { $count } failed

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
key-edit = edit tags and rating
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

## Other boorus

explore-collection = Collection
boorus-search = Search boorus
boorus-sites = Sites
boorus-site = Site
boorus-site-remove = Remove { $site }
boorus-address = Address
boorus-api = Kind of site
boorus-account = Account
boorus-account-forget = Forget
boorus-account-again = Enter it again
boorus-login = Username
boorus-user-id = User ID
boorus-key = API key
boorus-password-hash = Password hash
boorus-dedupe = Hide duplicates
boorus-known = Already here
boorus-extra-tags = Extra tags
boorus-save-label = Save the selected posts
boorus-save = Save { $count }
boorus-open-site = Open on site
boorus-also = Also on
boorus-score = Score
boorus-bad-address = Enter the site’s address, like https://booru.example.
boorus-need-account = Enter both the username (or user ID) and the key.
boorus-too-long = That’s longer than a site takes.
boorus-saving = Saving…

## Library imports

library-title = Import from another booru
library-lead = Moves a whole library here from szurubooru, Shimmie2, Danbooru, Stash, Blombooru, Monbooru or LANBooru: every post with its tags, rating, source, date and your likes, plus aliases, implications, tag trees, and pools or albums as tags. Your server does the work, so you can close this app. Importing again adds what’s new, and a post you already have only gains tags. Aliases and implications come only when an administrator imports. Your key stays on the server while the import runs and is erased when it ends.
library-left-out = Stays behind: comments, other users and their accounts, notes on images, titles and descriptions, site-wide scores and the order inside pools. StarBooru has no place for them.
library-source = Import from
library-hint-szurubooru = Log in with your username and a login token from your account’s Login tokens tab. Without them, you get what guests can see.
library-hint-shimmie2 = Needs the Ouroboros API turned on in Shimmie’s Extension Manager, and an API key if guests can’t see everything. Its aliases and auto-tag rules come across if they’re public, except those of several words. Its pools can’t be read: Shimmie has no API for them.
library-hint-danbooru = Your username and the API key from your profile. Without them, you get what guests can see, and your favourites stay behind. On large sites this means millions of posts, so use it on your own instance.
library-hint-stash = Scenes and images, oldest first. The API key is in Settings › Security. Stash has no safety ratings, so every post gets the rating you choose below. Performers become character tags, studios artist tags, and a rating of 60 or more becomes a like.
library-hint-blombooru = The API key from your account. Aliases and implications need an admin’s key on that side; tag-pattern implications don’t come across.
library-hint-monbooru = A token with read access, if the site uses them. Every gallery on the server comes across. Its aliases can’t be read: Monbooru has no API for them.
library-hint-lanbooru = No account needed: LANBooru’s reads are open. It has no aliases, implications or albums.
library-unrated = Rating for posts without a defining tag
library-check = Check
library-checking = Checking…
library-start = Import everything
library-found = Found { $count } posts.
library-found-some = The site answers, but doesn’t say how many posts it has.
library-no-rules = Aliases and implications will stay behind, because only an administrator can import them here.
library-full = Up to five imports can wait or run at once. Start this one when another ends.
library-bad-address = That address isn’t valid. Write it like http://192.168.1.4:8080.
library-need-login = This site needs the username along with the key.
library-need-key = This site needs the key along with the username.
library-list = Imports
library-state-queued = Waiting
library-state-rules = Reading aliases and implications
library-state-posts = Importing posts
library-state-groups = Reading pools and likes
library-state-done = Finished
library-state-stopped = Stopped
library-state-failed = Failed
library-count = { $done } done
library-count-of = { $done } of { $total }
library-count-merged = { $count } already here
library-count-failed = { $count } failed
library-count-rules = { $count } aliases and implications
library-stop = Stop
library-retry = Import again
library-remove = Remove from list
library-show-failed = Show failed
library-account-again = The key was erased when this import ended. Enter it again, or leave it empty to read what guests can see.
ago-now = just now
ago-minutes =
    { $count ->
        [one] 1 minute ago
       *[other] { $count } minutes ago
    }
ago-hours =
    { $count ->
        [one] 1 hour ago
       *[other] { $count } hours ago
    }
ago-yesterday = yesterday
ago-days = { $count } days ago

## Share links

share-title =
    { $count ->
        [one] Share this post
       *[other] Share { $count } posts
    }
share-lead =
    { $count ->
        [one] Anyone with the link can see this post without an account. Its tags, votes and details about you stay hidden.
       *[other] Anyone with the link can see these posts without an account. Their tags, votes and details about you stay hidden.
    }
share-name = Title (optional)
share-name-placeholder = e.g. Holiday photos
share-lifetime = Link works for
share-until-removed = Until removed
share-days =
    { $count ->
        [one] 1 day
       *[other] { $count } days
    }
share-create = Create link
share-ready = Link ready. Anyone who has it can open it.
share-link = Share link
share-copy = Copy link
share-copied = Copied
share-copy-failed = Couldn't copy the link. It's selected, so copy it from your keyboard or menu.
share-native = Share via…
share-done = Done
share-close = Close
share-posts =
    { $count ->
        [one] 1 post
       *[other] { $count } posts
    }
share-error-none = Pick at least one post to share.
share-error-many = A link can hold at most { $max } posts.
share-error-title-long = The title can be at most { $max } characters.
share-error-title-control = The title can't contain tabs or line breaks.
share-error-days = A link works for 1 to { $max } days, or until removed.
shares-title = Shared links
shares-lead = Links you made with Share. Removing one stops it working; a file someone already opened can keep loading for a few hours.
shares-none = No links yet. Use Share on a post, or select posts in the gallery.
shares-untitled = Untitled link
shares-made = Made { $when }
shares-views =
    { $count ->
        [0] Never opened
        [one] Opened once
       *[other] Opened { $count } times
    }
shares-expired = Expired
shares-no-expiry = Never expires
shares-expired-on = Expired { $date }
shares-expires-on = Expires { $date }
shares-expires-minutes =
    { $count ->
        [one] Expires in 1 minute
       *[other] Expires in { $count } minutes
    }
shares-expires-hours =
    { $count ->
        [one] Expires in 1 hour
       *[other] Expires in { $count } hours
    }
shares-expires-days =
    { $count ->
        [one] Expires in 1 day
       *[other] Expires in { $count } days
    }
shares-delete = Remove link
shares-delete-named = Remove the link { $name }
shared-missing-title = This link doesn't work
shared-missing-body = It may have expired, or the person who shared it removed it.
shared-from = Shared from { $server }
shared-until = available until { $date }
shared-empty = Nothing here any more.
shared-open = Item { $n }
shared-previous = Previous
shared-next = Next
