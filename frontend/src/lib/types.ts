// Types mirrored by hand from crates/raidian/proto/auth.proto and repository.proto.
// Keep in sync when the .proto changes.

export interface UserProfile {
  id: string;
  username: string;
  email: string;
  display_name: string;
  avatar_url: string;
  bio: string;
  is_admin: boolean;
  created_at: number;
  updated_at: number;
}

export interface AuthResponse {
  token: string;
  user: UserProfile;
}

export interface LoginRequest {
  username_or_email: string;
  password: string;
}

export interface RegisterRequest {
  username: string;
  email: string;
  password: string;
  display_name: string;
}

export interface GithubOauthRequest {
  code: string;
  state: string;
}

export interface ZixiaoOauthRequest {
  code: string;
  state: string;
  redirect_uri: string;
}

export interface ZixiaoCloudClientConfig {
  client_id: string;
  base_url: string;
}

export interface AuthProviders {
  local: boolean;
  github: boolean;
  zixiao_cloud: ZixiaoCloudClientConfig | null;
}

export interface AuthConfigResponse {
  deployment_mode: 'self-hosted' | 'saas';
  providers: AuthProviders;
}

export interface Repository {
  id: string;
  owner_id: string;
  owner_username: string;
  name: string;
  full_name: string;
  description: string;
  is_private: boolean;
  default_branch: string;
  created_at: number;
  updated_at: number;
}

export interface CreateRepositoryRequest {
  name: string;
  description: string;
  is_private: boolean;
}

export interface ApiErrorBody {
  error: string;
  message: string;
}
