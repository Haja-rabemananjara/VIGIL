type Dictionary = Record<string, string>;

const en: Dictionary = {
    "app.name": "VIGIL",
    "common.loading": "Loading...",
    "common.error": "Something went wrong",
    "common.retry": "Retry",
    "common.cancel": "Cancel",
    "common.confirm": "Confirm",

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

    "auth.error.invalidCredentials": "Invalid email or password",
    "auth.error.emailTaken": "This email is already in use",
    "auth.error.passwordTooShort": "Password must be at least 8 characters",

    "incident.state.open": "Open",
    "incident.state.acknowledged": "Acknowledged",
    "incident.state.escalated": "Escalated",
    "incident.state.resolved": "Resolved",

    "incident.severity.low": "Low",
    "incident.severity.medium": "Medium",
    "incident.severity.high": "High",
    "incident.severity.critical": "Critical",

    "app.shell.noTeamsYet": "No teams yet — create or join one to get started",

    "user.profile": "Profile",

    "onboarding.welcome": "Welcome",
    "onboarding.subtitle": "You're not part of any team yet. Get started below.",
    "onboarding.create.title": "Create a team",
    "onboarding.create.desc": "Start a new workspace and invite your teammates.",
    "onboarding.create.action": "Create a team",
    "onboarding.join.title": "Join a team",
    "onboarding.join.desc": "Have an invitation code? Join an existing team.",
    "onboarding.join.action": "Enter a code",
};

const dict: Dictionary = en;

export function t(key: string): string {
    return dict[key] ?? key;
}