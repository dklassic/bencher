use std::{fmt, str::FromStr};

use oso::{PolarClass, PolarValue, ToPolar};

use crate::{
    CREATE_PERM, CREATE_ROLE_ASSIGNMENTS_PERM, DELETE_PERM, DELETE_ROLE_ASSIGNMENTS_PERM,
    EDIT_PERM, LIST_ROLE_ASSIGNMENTS_PERM, MANAGE_PERM, UPDATE_ROLE_ASSIGNMENTS_PERM, VIEW_PERM,
};

pub const MEMBER_ROLE: &str = "member";
pub const LEADER_ROLE: &str = "leader";

#[derive(Debug, Clone, PolarClass)]
pub struct Organization {
    #[polar(attribute)]
    pub uuid: String,
}

#[derive(Debug, Clone, Copy)]
pub enum Role {
    Member,
    Leader,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Member => MEMBER_ROLE,
                Self::Leader => LEADER_ROLE,
            }
        )
    }
}

impl ToPolar for Role {
    fn to_polar(self) -> PolarValue {
        PolarValue::String(self.to_string())
    }
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            MEMBER_ROLE => Ok(Role::Member),
            LEADER_ROLE => Ok(Role::Leader),
            _ => Err(s.into()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Permission {
    View,
    Create,
    Edit,
    Delete,
    Manage,
    CreateRoleAssignments,
    ListRoleAssignments,
    UpdateRoleAssignments,
    DeleteRoleAssignments,
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::View => VIEW_PERM,
                Self::Create => CREATE_PERM,
                Self::Edit => EDIT_PERM,
                Self::Delete => DELETE_PERM,
                Self::Manage => MANAGE_PERM,
                Self::CreateRoleAssignments => CREATE_ROLE_ASSIGNMENTS_PERM,
                Self::ListRoleAssignments => LIST_ROLE_ASSIGNMENTS_PERM,
                Self::UpdateRoleAssignments => UPDATE_ROLE_ASSIGNMENTS_PERM,
                Self::DeleteRoleAssignments => DELETE_ROLE_ASSIGNMENTS_PERM,
            }
        )
    }
}

impl ToPolar for Permission {
    fn to_polar(self) -> PolarValue {
        PolarValue::String(self.to_string())
    }
}
