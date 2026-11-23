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
