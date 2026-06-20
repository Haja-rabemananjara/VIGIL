/**
 * Minimal i18n layer.
 *
 * Every user-facing string flows through t('key').
 * We will add FR dictionary and language switching.
 *
 * Keys follow the convention "scope.subscope.element" (e.g. "auth.signin.title").
 * If a key is missing from the dictionary, it returns the key itself, visible
 * in dev as a bug indicator, harmless in prod.
 */

type Dictionary = Record<string, string>;

const en: Dictionary = {
    // Generic
    "app.name": "VIGIL",
    "common.loading": "Loading...",
    "common.error": "Something went wrong",
    "common.retry": "Retry",
    "common.cancel": "Cancel",
    "common.confirm": "Confirm",

    // Auth
    "auth.signin.title": "Sign in to VIGIL",
    "auth.signin.email": "Email",
    "auth.signin.password": "Password",
    "auth.signin.submit": "Sign in",
    "auth.signin.switchToSignup": "Don't have an account? Sign up",

    "auth.signup.title": "Create your VIGIL account",
    "auth.signup.email": "Email",
    "auth.signup.password": "Password",
    "auth.signup.displayName": "Display name",
    "auth.signup.submit": "Sign up",
    "auth.signup.switchToSignin": "Already have an account? Sign in",

    "auth.signout.label": "Sign out",

    // Errors (matched against AppError codes from the Rust server)
    "auth.error.invalidCredentials": "Invalid email or password",
    "auth.error.emailTaken": "This email is already in use",
    "auth.error.passwordTooShort": "Password must be at least 8 characters",
};

// Single active dictionary.
const dict: Dictionary = en;

/**
 * Translate a key. Returns the key itself if not found, making missing keys
 * visible in the UI rather than silently shipping blank text.
 */
export function t(key: string): string {
    return dict[key] ?? key;
}