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
  "onboarding.myTeams": "Your teams",
  "onboarding.myTeams.desc": "Incident management will be available in a future update.",

  "teams.create.dialogTitle": "Create a new team",
  "teams.create.dialogDesc": "Give your team a name to get started.",
  "teams.create.nameLabel": "Team name",
  "teams.create.namePlaceholder": "e.g. Team front",
  "teams.create.submit": "Create",
  "teams.create.cancel": "Cancel",
  "teams.create.error.empty": "Team name is required",

  "teams.join.dialogTitle": "Join a team",
  "teams.join.dialogDesc": "Enter the invitation code shared by your team manager.",
  "teams.join.codeLabel": "Invitation code",
  "teams.join.codePlaceholder": "e.g. A7X9K2MP",
  "teams.join.submit": "Join",
  "teams.join.cancel": "Cancel",
  "teams.join.error.empty": "Invitation code is required",
  "teams.join.success": "You joined the team!",

  "teams.invite.dialogTitle": "Invite to team",
  "teams.invite.dialogDesc": "Share this code with people you want to invite.",
  "teams.invite.generate": "Generate code",
  "teams.invite.copy": "Copy",
  "teams.invite.copied": "Copied!",
  "teams.invite.close": "Close",
  "teams.invite.button": "Invite",
};

const dict: Dictionary = en;

export function t(key: string): string {
  return dict[key] ?? key;
}
