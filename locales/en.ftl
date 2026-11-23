# StarBooru UI text, English (the reference catalogue).

app-name = StarBooru

## Shared

notfound-title = Nothing here
notfound-body = There is no page at { $path }.
notfound-back = Back to the gallery
notice-close = Close

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
settings-people = People
settings-people-hint = The People card in the viewer. Off hides it; nothing is deleted
settings-appearance = Appearance
settings-theme = Theme
theme-system = System
theme-light = Light
theme-dark = Dark
settings-language = Language
settings-language-system = System
lang-english = English
lang-italian = Italiano
settings-accent = Accent colour
settings-accent-custom = Custom accent colour
accent-no-hue = Greys have no hue to build the colours from. Pick a stronger colour.
accent-violet = Violet
accent-pink = Pink
accent-coral = Coral
accent-amber = Amber
accent-green = Green
accent-teal = Teal
accent-blue = Blue
accent-red = Red
settings-hand = Hand
settings-hand-hint = Puts the side navigation on tablets and laptops, the editor's tools and the buttons of the Upload page on the side of the hand you use.
hand-either = Either
hand-left = Left
hand-right = Right
settings-saver = Battery saver
settings-saver-hint = While it's on, videos don't start by themselves, the next post isn't loaded ahead, and animations are cut short. Automatic turns it on when the battery is at 20% or less and not charging, or when the device is saving data.
settings-saver-now = On now: the battery is low or the device is saving data.
saver-auto = Automatic
saver-always = Always
saver-never = Never
settings-effects = Effects
settings-effects-hint = Reduced shows plain surfaces in place of the frosted glass, and leaves out the entrance of new thumbnails and the glow behind a picture. Automatic does that once scrolling stutters on this device.
settings-effects-now = Reduced now: scrolling was stuttering on this device. Choose Automatic again to measure it once more.
effects-auto = Automatic
effects-full = Full
effects-lite = Reduced
settings-originals = Always open originals in the viewer
settings-originals-hint = Big images otherwise open as a smaller copy, which loads faster and uses less data. The original still opens when you zoom in, or when the screen has more pixels than the copy.
settings-flush = Remove the spaces and rounded corners between images
settings-flush-hint = Thumbnails sit edge to edge, with square corners.
settings-keep-original = Keep the original when saving an edit
settings-keep-original-hint = The edited picture or video is saved as a new post beside it. Otherwise the original goes to the recycle bin.
settings-gestures = Gestures
settings-gestures-hint = Swipe a thumbnail sideways to act on it at once. Hold a thumbnail or a tag, or right click it, for all it can do.
settings-swipe-left = Swipe left
settings-swipe-right = Swipe right
swipe-off = Nothing
swipe-delete = Move to recycle bin
admin-server-title = Server
settings-admin-note = Server stats, accounts and settings
settings-help-legal = Help and legal
settings-help-note = Query syntax, examples and keyboard shortcuts
settings-privacy-note = What is stored, where, and for how long
settings-terms-note = The rules for using this server
settings-role = Role
role-admin = admin
role-user = user
settings-storage = Uploads
usage-of = { $used } of { $quota }
signin-title = Ways to sign in
signin-password-set = Set
signin-password-none = Not set
signin-provider = Single sign-on
signin-times = Linked { $linked } · last used { $used }
signin-linked = Linked { $linked }
signin-last = This is your only way to sign in. Set a password before unlinking it.
signin-link = Link a { $provider } account
signin-unlink = Unlink
signin-unlink-named = Unlink { $name }
oidc-back-settings = Back to Settings
password-current = Current password
password-new = New password
password-too-short = At least { $count } characters.
password-repeat = Repeat the new password
password-mismatch = The passwords don't match.
password-wrong-current = The current password is wrong.
password-changed = Password changed. Your other devices were signed out.
password-changing = Changing…
password-change = Change password
password-none = This account has no password yet. Set one to sign in without the provider too.
password-set = Set password
sessions-title = Signed-in devices
sessions-unknown = Unknown client
sessions-this-device = This device
sessions-times = Active { $active } · signed in { $created }
sessions-revoke-others = Sign out everywhere else
sessions-revoke = Sign out this device
keys-title = API keys
keys-lead = Let scripts and other tools use StarBooru as you. A key can't change your password, sign-ins or keys.
keys-copy-now = Copy the key “{ $name }” now. It won't be shown again.
keys-token = New API key
keys-copy = Copy
keys-copied = Copied
keys-copy-failed = Couldn't copy the key. It's selected, so copy it from your keyboard or menu.
keys-none = No API keys yet.
keys-read-only = Read-only
keys-full = Full access
keys-expired = Expired
keys-used = Used { $when }
keys-never-used = Never used
keys-no-expiry = Never expires
keys-expired-on = Expired { $date }
keys-expires-on = Expires { $date }
keys-name = Name
keys-name-placeholder = What will use it, e.g. backup script
keys-access = Access
keys-lifetime = Expires
keys-never-expires = Never
keys-days =
    { $count ->
        [one] 1 day
       *[other] { $count } days
    }
keys-create = Create key
keys-delete = Remove key
keys-delete-named = Remove the key { $name }
keys-error-name = Give the key a name.
keys-error-long = The name is longer than { $max } characters.
keys-error-control = The name can't hold control characters.
keys-error-days = A key can last 1 to { $max } days.
about-title = About StarBooru
about-made-by = Made by
about-made-by-name = FilipNone
about-icons = Icons
about-icons-names = MingCute, Material Symbols, Solar, Fluent UI
about-lettering = Logo lettering
about-lettering-name = M PLUS Rounded 1c
about-version = App version
about-github = GitHub
about-credits = All credits
about-credits-note = Everyone whose work StarBooru uses, with their licenses

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

explore-nothing = Nothing to explore yet
explore-nothing-body = Upload images or videos, and they show up here.
explore-everything = Explore everything
nothing-matches = Nothing matches “{ $query }”.
explore-on-this-day = On this day
explore-no-memories = Nothing was added on this date in earlier years.
explore-years-ago = { $count ->
    [one] 1 year ago
   *[other] { $count } years ago
}
explore-random = Random picks
explore-shuffle = Shuffle
explore-search = Search Explore

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
dup-some-of = { $count ->
        [one] { $shown } of 1 post
       *[other] { $shown } of { $count } posts
    }
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
download-failed = The download didn't finish. Check your connection and try again.
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
people-turned-off = People were turned off in Settings. This card goes away once you save or cancel.
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

## Privacy Policy and Terms of Service

legal-back-sign-in = Back to sign-in
legal-back-settings = Back to Settings
legal-server = For { $name } at { $address }
legal-server-address = For the server at { $address }
legal-section-link = Link to this section
privacy-lead = StarBooru is software that runs on a server chosen by you or someone you trust. Nothing you store is sent to the StarBooru project, or to anyone else except as this page says.
privacy-responsible = Who is responsible
privacy-responsible-body = The person or organisation that runs this server (the operator) decides who may sign up and is responsible for the data on it. Questions about your data go to them.
privacy-server = What the server stores
privacy-server-account = Your account: username, role and a salted Argon2id hash of your password. The password itself is never stored. If you sign in through the operator’s identity provider, the server keeps that provider’s id for you and the username or email it reports, to recognise you next time.
privacy-server-sessions = Your sessions: a hash of each sign-in token, the kind of device it came from (for example “Firefox on Linux” or “StarBooru app on Android”), when it was created and last used, and when it expires. API keys you create are kept the same way: a hash of the key, its first characters to tell it apart, its name, and when it was last used.
privacy-server-files = Your uploads: each file with its size, dimensions, checksum and a visual fingerprint for the duplicate finder, and what the server reads from it: the date in the file, its main colours and any text in a still image. The server notes who uploaded each post.
privacy-server-content = What you add: tags, the people marked in a post, ratings, sources, translations of comic pages, votes with when you cast them, and saved searches.
privacy-server-shares = For each share link you make: the posts in it, its title, when it expires, and how many times and when it was last opened (not by whom).
privacy-server-sites = For other sites: the searches you save on other boorus, the accounts you save for them (the login as you typed it, the key encrypted with the server’s secret), the addresses you import, kept for 7 days after they finish, and each library import with its counts until you remove it.
privacy-server-limits = To slow down password guessing, the server counts failed sign-ins per account and per network address, and sign-ups per address. These counts are kept in memory for at most an hour and never written to disk.
privacy-device = What stays on your device
privacy-device-storage = This app keeps one entry in the browser’s local storage: the server address, your sign-in token, your username and role, your preferences (such as the theme, the language, the gallery’s layout and the boorus you added by address), and where the gallery was in your last 30 searches.
privacy-device-sign-in = While you sign in through an identity provider, the tab keeps a one-time handle in its session storage, removed when you come back.
privacy-device-sign-out = Signing out removes the token, your username and role, and the searches the gallery remembered. The server address and the preferences stay until you clear this site’s data in the browser.
privacy-device-cache = StarBooru sets no cookies. Pictures and videos go into the browser’s cache, as on any website, and everything else the app loads stays in the tab’s memory until you close it.
privacy-not-done = What StarBooru does not do
privacy-not-done-tracking = No analytics, advertising or tracking, and no third-party services beyond an identity provider or an AI provider the operator chooses to set up. This app loads its code, styles and icons from this server alone.
privacy-not-done-contacts = The server contacts other sites only when you search, browse or import from another booru, import from a URL or sign in through the operator’s identity provider, and when an administrator installs a local AI model, which it downloads from where the model is published. Its requests name StarBooru as the client and carry none of your details, except the account you saved for that site, which goes to that site alone.
privacy-not-done-boorus = When you browse another booru here, your browser loads that site’s pictures from the site itself, without saying which page asked for them, so the site sees your network address as it would on its own pages. Pictures from sites that serve them only to their own pages come through this server instead.
privacy-not-done-ai = If an administrator sets up an AI provider, it gets thumbnails of posts to suggest tags (new uploads too, when automatic tagging is on), the text read in an image when you translate it, and the image itself when its comic or manga panels are translated, by hand or for new uploads when automatic translation is on. A provider on the operator’s own network keeps them there; local models run on the server, and nothing leaves it.
privacy-not-done-library = While a whole library is being imported, the key you gave for that site is kept on the server, and in any backup taken meanwhile; it’s erased when the import ends.
privacy-not-done-logs = File contents, the text read in images, passwords and tokens are never written to the server’s logs.
privacy-who-sees = Who can see your posts
privacy-who-sees-body = Everyone with an account on this server can see the posts on it, and download the original files with anything they hold, such as where a photo was taken. A post shows its score, not who voted. Media links handed to the app are signed and expire, so a copied link stops working within 12 hours. If you make a share link, anyone who has it can see the posts in it, without their tags, until it expires or you remove it.
privacy-keeping = Keeping and deleting
privacy-keeping-bin = Deleted posts go to the recycle bin. From there an administrator, or whoever uploaded them, can erase them with their files, and the operator can have the bin emptied by itself after a set number of days.
privacy-keeping-sessions = Sessions end when you sign out, after 90 days, or when you sign out other devices in Settings. Changing your password signs out every other device; API keys keep working until you remove them or they expire.
privacy-keeping-admins = Administrators of this server see each account’s username, role, when it joined and was last active, and how much it has uploaded. They can set a storage limit, reset a forgotten password, disable an account or delete it. To delete your account, ask one of them: that removes its sessions, keys, votes and saved searches, while its posts stay.
privacy-keeping-backups = Administrators can back up the database, by hand or on a schedule. A backup holds everything listed above except the files, so something erased here stays in older backups until they are deleted. The operator may keep other copies on their own schedule.
terms-lead = These terms cover your use of this StarBooru server. The operator who runs it may add to them.
terms-account = Your account
terms-account-keys = Keep your password and API keys to yourself, and sign out devices you no longer use (Settings › Signed-in devices).
terms-account-admins = The first account on a server is its administrator. Administrators can manage tags, aliases and implications, empty the recycle bin, change the server’s settings, set up AI tools, make backups, and manage accounts: roles, storage limits, password resets, disabling and deleting.
terms-content = Your content
terms-content-rights = Upload and import only what you have the right to keep and to share with the people on this server and anyone you send a share link to, and what is legal where the server runs.
terms-content-yours = You keep every right to what you upload. You let the server store it, make thumbnails and previews, and show it to the accounts on this server and to the people you share it with.
terms-content-originals = Originals are kept byte for byte. Rate posts honestly, so that rating filters work for everyone.
terms-fair = Fair use
terms-fair-access = Don't try to reach other people's accounts or data, or to get around the server's limits.
terms-fair-load = Don't overload the server, or the sites it reaches for you, with automated requests.
terms-warranty = No warranty
terms-warranty-body = StarBooru is provided as is, without warranty of any kind. Keep your own copy of anything you can't afford to lose. The operator can suspend accounts that break these terms and can change them; the current version is always on this page.

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
library-found =
    { $count ->
        [one] Found 1 post.
       *[other] Found { $count } posts.
    }
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
library-count-rules =
    { $count ->
        [one] 1 alias or implication
       *[other] { $count } aliases and implications
    }
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
ago-days =
    { $count ->
        [one] 1 day ago
       *[other] { $count } days ago
    }

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
share-pick-day = Until a date
share-day = Last day the link works
share-day-missing = Pick the last day the link works.
share-day-soon = Pick tomorrow or a later day.
share-day-late = Pick { $last } or earlier: { $max } days at most.
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

## Administration

admin-only = Only admins can open this page.
admin-tab-overview = Overview
admin-tab-accounts = Accounts
admin-tab-backups = Backups
admin-tab-server = Server
admin-on = On
admin-off = Off
admin-posts = Posts
admin-images = Images
admin-animations = Animations
admin-videos = Videos
admin-bin = In the recycle bin
admin-tags = Tags
admin-accounts = Accounts
admin-media = Media on disk
admin-database = Database size
admin-uploads-title = Uploads, last 30 days
admin-uploads-total =
    { $count ->
        [one] 1 upload in the last 30 days
       *[other] { $count } uploads in the last 30 days
    }
admin-running = Started
admin-upload-limit = Largest upload
admin-bin-size = Recycle bin size
admin-disabled-accounts = Disabled accounts
admin-video-thumbs = Video thumbnails (ffmpeg)
admin-url-import = Import from URLs
admin-accounts-lead = Everyone with an account on this server, what they uploaded and when they were last active.
admin-disabled = Disabled
admin-you = You
admin-active = active { $when }
admin-never-active = never active
admin-joined = joined { $date }
admin-manage = Manage
admin-manage-named = Manage { $name }
admin-role-saved = Role changed.
admin-limit = Storage limit
admin-limit-none = No limit
admin-limit-hint = In GB, or with its unit, such as 500 MB. Uploads that would go past it are refused. Leave it empty for no limit.
admin-limit-save = Set limit
admin-limit-saved = Storage limit saved.
admin-limit-invalid = Type a size, such as 5 GB or 500 MB, or leave it empty.
admin-password-new = New password
admin-password-hint = For someone who forgot theirs. Signs them out everywhere; their API keys keep working.
admin-password-reset = Reset
admin-password-reset-done = Password reset. Tell them the new one.
admin-disable = Disable account
admin-enable = Enable account
admin-disabled-done = Account disabled and signed out.
admin-enabled-done = Account enabled.
admin-delete = Delete account
admin-delete-confirm = Delete { $name } for good?
admin-delete-note = Deleting removes the account, its votes, saved searches and keys. Its posts stay.
admin-deleted = Account { $name } deleted.
admin-server-lead = These apply at once and override the server's environment variables.
admin-name = Server name
admin-name-hint = Shown on the sign-in page.
admin-name-error = Give the server a name of 1 to { $max } characters.
admin-registration = Anyone can create an account
admin-registration-hint = Off: only the accounts that exist can sign in.
admin-trash-days = Empty the recycle bin after (days)
admin-trash-days-hint = Posts older than this in the bin are deleted for good, hourly. 0 keeps them until emptied by hand.
admin-trash-days-error = Type a number of days from 0 to { $max }.
admin-auto-tags = Tag new posts with what the file shows
admin-auto-tags-hint = Meta tags like highres, lowres, wide_image, video, sound and long_video. Each post is tagged once, so a tag you remove stays removed. Turning this on also tags older posts at the next server start.
admin-save = Save
admin-saved = Saved
admin-bin-open = Open the recycle bin
admin-bin-empty-confirm =
    { $count ->
        [one] Delete 1 post for good?
       *[other] Delete { $count } posts for good?
    }
admin-ai-title = AI
admin-ai-unavailable = Local AI can't start on this server.
admin-ai-unavailable-api = Each tool can still use an API instead.
admin-ai-hw-threads =
    { $count ->
        [one] 1 thread
       *[other] { $count } threads
    }
admin-ai-hw-memory = { $total } RAM
admin-ai-tagging = Tag suggestions
admin-ai-translation = Translation
admin-ai-comics = Comic translation
admin-ai-mode-off = Off
admin-ai-mode-local = Local
admin-ai-mode-api = API
admin-ai-speed = Speed
admin-ai-preset-fast = Fast
admin-ai-preset-balanced = Balanced
admin-ai-preset-accurate = Accurate
admin-ai-presets-off = Not available on this machine: { $presets }
admin-ai-state-ready = Ready
admin-ai-state-preparing = Preparing…
admin-ai-state-downloading = Downloading { $percent }%
admin-ai-state-failed = Failed
admin-ai-models = Models
admin-ai-no-memory = Not enough memory
admin-ai-gpu = GPU
admin-ai-high-accuracy = High accuracy
admin-ai-in-use = In use
admin-ai-use = Use
admin-ai-install = Install
admin-ai-uninstall = Uninstall
admin-ai-url = Address
admin-ai-model = Model
admin-ai-key = API key
admin-ai-key-set = Saved
admin-ai-key-none = None
admin-ai-key-clear = Remove key
admin-ai-test = Test
admin-ai-testing = Testing…
admin-ai-test-ok = Answered in { $time }
admin-ai-test-failed = The API didn't answer: { $reason }.
admin-ai-error-url = Enter the API's address.
admin-ai-error-url-scheme = The address starts with http:// or https://.
admin-ai-error-model = Enter the name of the model to use.
admin-ai-error-long = The address, model and key can each be up to { $max } characters, on one line.
admin-ai-autotag = Tag new uploads automatically
admin-ai-auto-translate = Translate new comics to
admin-ai-auto-translate-hint = Language code, such as en. Empty: don't.
admin-ai-translate-invalid = That isn't a language code. Type one such as en or pt-BR, or leave it empty.
backups-title = Database backups
backups-lead = A backup holds the database: posts, tags, accounts and settings. Media files stay in the data folder, so copy that folder as well.
backups-unavailable = This server can't make backups: the PostgreSQL client (pg_dump) is missing. Install it, or point STARBOORU_PG_DUMP at it.
backups-now = Back up now
backups-running = Backing up…
backups-made = Backup made.
backups-none = No backups yet.
backups-automatic = Automatic
backups-download-named = Download backup { $name }
backups-delete = Delete backup
backups-delete-named = Delete backup { $name }
backups-schedule-title = Automatic backups
backups-every = Every (days)
backups-every-hint = 0 turns automatic backups off.
backups-every-error = From 0 to { $max } days.
backups-keep = Keep
backups-keep-hint = Older automatic backups are removed. Backups made by hand stay until you delete them.
backups-keep-error = From 1 to { $max }.
backups-restore-title = Restoring
backups-restore = Stop StarBooru, then load the backup into its database with pg_restore:
backups-restore-note = Start StarBooru again afterwards. The deployment guide has the full steps.

## Sizes, and the viewer's and tag pages' second thoughts

size-b = { $size } B
size-kb = { $size } KB
size-mb = { $size } MB
size-gb = { $size } GB
size-tb = { $size } TB
sure-named = Sure? { $action }
post-unsaved = Save or cancel your changes first.
post-neighbours-failed = Couldn't find the posts around this one: { $reason }
post-media-failed = The file didn't load. Its link may have expired.
people-remove-named = Remove { $person }
ai-suggest-people-replace = Replace the people typed?

## Failures in words: the server's side, searches, saved searches, downloads and uploads

error-server-fault = Something went wrong on { $server }. Try again in a minute. If it keeps happening, tell whoever runs the server; it has the details in its log.
error-storage-full = { $server } has run out of disk space. Whoever runs it needs to free some before anything more can be saved.
error-proxy-too-large = { $server } turned this away as too large before StarBooru saw it. Whoever runs it can raise the upload limit of the web server in front of it.
error-unexpected-answer = { $server } gave an answer StarBooru didn't expect ({ $status }). Check the address, or try again later.
query-error-terms = Too many search terms. Use { $max } at most.
query-error-rating = Unknown rating “{ $value }”. Use g, s, q or e.
query-error-kind = Unknown type “{ $value }”. Use image, animation or video.
query-error-status = Unknown status “{ $value }”. Use active or deleted.
query-error-order = Unknown order “{ $value }”. Use new, old, score, size, pixels, longside, tags, updated or random, and add _asc to reverse it.
query-error-text-empty = text: needs some words, such as text:hello or text:"two words".
query-error-text-long = text: is too long. Keep it to { $max } characters.
query-error-figure-empty = { "{}" } needs tags inside, such as { "{" }girl blonde_hair boots{ "}" }.
query-error-figure-kinds = A { "{…}" } term names two kinds. Use one of girl, boy or other.
query-error-date = date:{ $value } isn't a date. Try date:2024-03, date:2023..2024 or date:>30d.
query-error-size = { $field }:{ $value } isn't a size. Try width:>1920, mpixels:>8 or ratio:16:9.
query-error-color = color:{ $value } isn't a colour. Try color:red or color:2060e0.
query-error-tag-long = A tag in the search is too long. Keep tags to { $max } characters.
query-error-tag-other = A tag in the search has a character no tag can have.
saved-error-no-name = Give the search a name.
saved-error-name-long = The name is too long. Keep it to { $max } characters.
saved-error-control = The name has a character it can't have.
saved-error-empty = There's no search to save.
saved-error-long = The search is too long to save. Keep it to { $max } characters.
download-expired = The link to this file has expired. Reload the page and try again.
upload-stopped-late = { $name } had already arrived, as post { $id }.
upload-drop-waiting =
    { $count ->
        [one] The file waits on the Upload page.
       *[other] { $count } files wait on the Upload page.
    }

## Other boorus, Settings and the administration, second pass

boorus-saved-as = Saved here as post #{ $id }
boorus-save-failed = Couldn't save it: { $reason }
boorus-post-missing = This address doesn't say which post to open.
boorus-account-left = An account for { $site } ({ $login }) is saved on the server, but the site isn't in this list.
search-list-failed = Couldn't read the saved searches: { $reason } Press to try again.
keys-done = Done
admin-delete-named = Delete account { $name }
admin-ai-uninstall-named = Uninstall { $name }
admin-ai-downloading-named = Downloading { $name }
