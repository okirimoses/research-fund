#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// Data Structures

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Researcher {
    id: u64,
    name: String,
    address: String,
    email: String,
    phone: String,
    owner: String,
    reputation_score: u64,
    total_points: u64,
    badges: Vec<String>,
    contributions: Vec<String>, // ids of contributions
    achievements: Vec<String>,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct ResearchProposal {
    id: u64,
    researcher_id: u64,
    title: String,
    description: String,
    methodology: String,
    milestones: Vec<u64>,
    funding_target: u64,
    current_funding: u64,
    stage: String,
    reviews: Vec<u64>,
    timeline: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Milestone {
    id: u64,
    description: String,
    required_funding: u64,
    deadline: String,
    status: String,
    proofs: Vec<u64>,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Review {
    id: u64,
    proposal_id: u64,
    reviewer: String,
    score: u64,
    comments: String,
    stake_amount: u64,
    verified: bool,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct ProofOfReproduction {
    id: u64,
    methodology_hash: String,
    results_hash: String,
    status: String, // "pending", "verified", "rejected"
}

// Message Enum
#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
enum Message {
    Success(String),
    Error(String),
    NotFound(String),
    InvalidPayload(String),
}

// Payload Structures
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct ResearcherPayload {
    name: String,
    address: String,
    email: String,
    phone: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct CreateProposalPayload {
    researcher_id: u64,
    title: String,
    description: String,
    methodology: String,
    funding_target: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct SubmitReviewPayload {
    proposal_id: u64,
    reviewer: String,
    score: u64,
    comments: String,
    stake_amount: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct FundProposalPayload {
    proposal_id: u64,
    funding_amount: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct CreateMilestonePayload {
    proposal_id: u64,
    description: String,
    required_funding: u64,
    deadline: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct VerifyMilestonePayload {
    proposal_id: u64,
    milestone_id: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct SubmitProofPayload {
    milestone_id: u64,
    methodology_hash: String,
    results_hash: String,
}

// Implement Storable and BoundedStorable for each struct
macro_rules! impl_storable {
    ($($t:ty),*) => {
        $(
            impl Storable for $t {
                fn to_bytes(&self) -> Cow<[u8]> {
                    Cow::Owned(Encode!(self).unwrap())
                }

                fn from_bytes(bytes: Cow<[u8]>) -> Self {
                    Decode!(bytes.as_ref(), Self).unwrap()
                }
            }

            impl BoundedStorable for $t {
                const MAX_SIZE: u32 = 1024;
                const IS_FIXED_SIZE: bool = false;
            }
        )*
    };
}

impl_storable!(
    Researcher,
    ResearchProposal,
    Milestone,
    Review,
    ProofOfReproduction,
    Message
);

// Memory Management
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static RESEARCHERS: RefCell<StableBTreeMap<u64, Researcher, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static PROPOSALS: RefCell<StableBTreeMap<u64, ResearchProposal, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static MILESTONES: RefCell<StableBTreeMap<u64, Milestone, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));

    static REVIEWS: RefCell<StableBTreeMap<u64, Review, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
    ));

    static PROOFS: RefCell<StableBTreeMap<u64, ProofOfReproduction, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5)))
    ));
}

// Utility function to generate unique ID
fn generate_id() -> u64 {
    ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow().get();
        counter
            .borrow_mut()
            .set(current_value + 1)
            .expect("Failed to increment counter");
        current_value + 1
    })
}

// Create Researcher Function
#[ic_cdk::update]
fn create_researcher(payload: ResearcherPayload) -> Result<Researcher, Message> {
    // Input validations
    if payload.name.trim().len() < 2 {
        return Err(Message::InvalidPayload(
            "Name must be at least 2 characters long".to_string(),
        ));
    }

    if payload.address.trim().len() < 5 {
        return Err(Message::InvalidPayload(
            "Address must be at least 5 characters long".to_string(),
        ));
    }

    // Email validation (basic regex)
    let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    if !email_regex.is_match(&payload.email) {
        return Err(Message::InvalidPayload("Invalid email format".to_string()));
    }

    // Phone validation
    let phone_regex = regex::Regex::new(r"^[0-9]{10,15}$").unwrap();
    if !phone_regex.is_match(&payload.phone) {
        return Err(Message::InvalidPayload(
            "Phone number must be 10-15 digits".to_string(),
        ));
    }

    // Check for duplicate researchers
    let existing_researchers =
        RESEARCHERS.with(|r| r.borrow().iter().map(|(_, v)| v).collect::<Vec<_>>());
    if existing_researchers
        .iter()
        .any(|researcher| researcher.email == payload.email || researcher.phone == payload.phone)
    {
        return Err(Message::InvalidPayload(
            "Researcher with this email or phone already exists".to_string(),
        ));
    }

    let id = generate_id();
    let caller = ic_cdk::api::caller();

    let researcher = Researcher {
        id,
        name: payload.name.trim().to_string(),
        address: payload.address.trim().to_string(),
        email: payload.email.trim().to_lowercase(),
        phone: payload.phone.replace(|c: char| !c.is_digit(10), ""),
        owner: caller.to_string(),
        reputation_score: 0,
        total_points: 0,
        badges: Vec::new(),
        contributions: Vec::new(),
        achievements: Vec::new(),
    };

    RESEARCHERS.with(|researchers| {
        researchers.borrow_mut().insert(id, researcher.clone());
    });

    Ok(researcher)
}

// Get Researcher by ID
#[ic_cdk::query]
fn get_researcher_by_id(researcher_id: u64) -> Result<Researcher, Message> {
    RESEARCHERS.with(|researchers| {
        researchers
            .borrow()
            .get(&researcher_id)
            .ok_or(Message::NotFound(format!(
                "Researcher with id={} not found",
                researcher_id
            )))
    })
}

// Get All Researchers
#[ic_cdk::query]
fn get_all_researchers() -> Result<Vec<Researcher>, Message> {
    let all_researchers = RESEARCHERS.with(|researchers| {
        researchers
            .borrow()
            .iter()
            .map(|(_, v)| v)
            .collect::<Vec<_>>()
    });

    if all_researchers.is_empty() {
        Err(Message::NotFound("No researchers found".to_string()))
    } else {
        Ok(all_researchers)
    }
}

// Get Researcher by Owner
#[ic_cdk::query]
fn get_researcher_by_owner() -> Result<Researcher, Message> {
    let caller = ic_cdk::api::caller();
    RESEARCHERS.with(|researchers| {
        researchers
            .borrow()
            .iter()
            .map(|(_, r)| r)
            .find(|researcher| researcher.owner == caller.to_string())
            .ok_or(Message::NotFound(format!(
                "Researcher with owner={} not found",
                caller
            )))
    })
}

// Create Proposal Function
#[ic_cdk::update]
fn create_proposal(payload: CreateProposalPayload) -> Result<ResearchProposal, Message> {
    // Validate inputs
    if payload.title.is_empty()
        || payload.description.is_empty()
        || payload.methodology.is_empty()
        || payload.funding_target == 0
    {
        return Err(Message::InvalidPayload(
            "All fields are required, and funding target must be > 0".to_string(),
        ));
    }

    // Check if researcher exists
    RESEARCHERS.with(|researchers| {
        if researchers.borrow().get(&payload.researcher_id).is_none() {
            return Err(Message::NotFound(format!(
                "Researcher with id={} not found",
                payload.researcher_id
            )));
        }

        let id = generate_id();
        let proposal = ResearchProposal {
            id,
            researcher_id: payload.researcher_id,
            title: payload.title,
            description: payload.description,
            methodology: payload.methodology,
            milestones: Vec::new(),
            funding_target: payload.funding_target,
            current_funding: 0,
            stage: "draft".to_string(),
            reviews: Vec::new(),
            timeline: serde_json::json!({
                "created_at": time()
            })
            .to_string(),
        };

        PROPOSALS.with(|proposals| {
            proposals.borrow_mut().insert(id, proposal.clone());
        });

        Ok(proposal)
    })
}

// Get Proposal by ID
#[ic_cdk::query]
fn get_proposal_by_id(proposal_id: u64) -> Result<ResearchProposal, Message> {
    PROPOSALS.with(|proposals| {
        proposals
            .borrow()
            .get(&proposal_id)
            .ok_or(Message::NotFound(format!(
                "Proposal with id={} not found",
                proposal_id
            )))
    })
}

// Get All Proposals
#[ic_cdk::query]
fn get_all_proposals() -> Result<Vec<ResearchProposal>, Message> {
    let all_proposals = PROPOSALS.with(|proposals| {
        proposals
            .borrow()
            .iter()
            .map(|(_, v)| v)
            .collect::<Vec<_>>()
    });

    if all_proposals.is_empty() {
        Err(Message::NotFound("No proposals found".to_string()))
    } else {
        Ok(all_proposals)
    }
}

// Get Proposals by Researcher ID
#[ic_cdk::query]
fn get_proposals_by_researcher_id(researcher_id: u64) -> Result<Vec<ResearchProposal>, Message> {
    let proposals = PROPOSALS.with(|proposals| {
        proposals
            .borrow()
            .iter()
            .map(|(_, p)| p)
            .filter(|proposal| proposal.researcher_id == researcher_id)
            .collect::<Vec<_>>()
    });

    if proposals.is_empty() {
        Err(Message::NotFound(format!(
            "No proposals found for researcher_id={}",
            researcher_id
        )))
    } else {
        Ok(proposals)
    }
}

// Submit Review Function
#[ic_cdk::update]
fn submit_review(payload: SubmitReviewPayload) -> Result<Review, Message> {
    // Validate inputs
    if payload.score == 0 || payload.comments.is_empty() || payload.stake_amount == 0 {
        return Err(Message::InvalidPayload(
            "Score, comments, and stake amount are required".to_string(),
        ));
    }

    // Check if proposal exists
    PROPOSALS.with(|proposals| {
        if proposals.borrow().get(&payload.proposal_id).is_none() {
            return Err(Message::NotFound(format!(
                "Proposal with id={} not found",
                payload.proposal_id
            )));
        }

        let id = generate_id();
        let review = Review {
            id,
            proposal_id: payload.proposal_id,
            reviewer: payload.reviewer.to_string(),
            score: payload.score,
            comments: payload.comments,
            stake_amount: payload.stake_amount,
            verified: false,
        };

        REVIEWS.with(|reviews| {
            reviews.borrow_mut().insert(id, review.clone());
        });

        Ok(review)
    })
}

// Get Review by ID
#[ic_cdk::query]
fn get_review_by_id(review_id: u64) -> Result<Review, Message> {
    REVIEWS.with(|reviews| {
        reviews
            .borrow()
            .get(&review_id)
            .ok_or(Message::NotFound(format!(
                "Review with id={} not found",
                review_id
            )))
    })
}

// Get Reviews by Proposal ID
#[ic_cdk::query]
fn get_reviews_by_proposal_id(proposal_id: u64) -> Result<Vec<Review>, Message> {
    let reviews = REVIEWS.with(|reviews| {
        reviews
            .borrow()
            .iter()
            .map(|(_, review)| review)
            .filter(|review| review.proposal_id == proposal_id)
            .collect::<Vec<_>>()
    });

    if reviews.is_empty() {
        Err(Message::NotFound(format!(
            "No reviews found for proposal_id={}",
            proposal_id
        )))
    } else {
        Ok(reviews)
    }
}

// Fund Proposal Function
#[ic_cdk::update]
fn fund_proposal(payload: FundProposalPayload) -> Result<ResearchProposal, Message> {
    // Validate inputs
    if payload.funding_amount == 0 {
        return Err(Message::InvalidPayload(
            "Funding amount must be > 0".to_string(),
        ));
    }

    // Check if proposal exists
    PROPOSALS.with(|proposals| {
        let mut proposals = proposals.borrow_mut();
        if let Some(mut proposal) = proposals.remove(&payload.proposal_id) {
            proposal.current_funding += payload.funding_amount;
            proposals.insert(payload.proposal_id, proposal.clone());
            Ok(proposal)
        } else {
            Err(Message::NotFound(format!(
                "Proposal with id={} not found",
                payload.proposal_id
            )))
        }
    })
}

// Create Milestone Function
#[ic_cdk::update]
fn create_milestone(payload: CreateMilestonePayload) -> Result<Milestone, Message> {
    // Validate inputs
    if payload.description.is_empty() || payload.required_funding == 0 {
        return Err(Message::InvalidPayload(
            "Description and required funding are required".to_string(),
        ));
    }

    // Check if proposal exists
    PROPOSALS.with(|proposals| {
        if proposals.borrow().contains_key(&payload.proposal_id) {
            let id = generate_id();
            let milestone = Milestone {
                id,
                description: payload.description,
                required_funding: payload.required_funding,
                deadline: payload.deadline,
                status: "pending".to_string(),
                proofs: Vec::new(),
            };

            MILESTONES.with(|milestones| {
                milestones.borrow_mut().insert(id, milestone.clone());
            });

            let mut proposals = proposals.borrow_mut();
            if let Some(mut proposal) = proposals.remove(&payload.proposal_id) {
                proposal.milestones.push(id);
                proposal.stage = "active".to_string();
                proposal.timeline = serde_json::json!({
                    "milestone_created_at": time()
                })
                .to_string();
                proposals.insert(payload.proposal_id, proposal.clone());
                Ok(milestone)
            } else {
                Err(Message::NotFound(format!(
                    "Proposal with id={} not found",
                    payload.proposal_id
                )))
            }
        } else {
            Err(Message::NotFound(format!(
                "Proposal with id={} not found",
                payload.proposal_id
            )))
        }
    })
}

// Get Milestone by ID
#[ic_cdk::query]
fn get_milestone_by_id(milestone_id: u64) -> Result<Milestone, Message> {
    MILESTONES.with(|milestones| {
        milestones
            .borrow()
            .get(&milestone_id)
            .ok_or(Message::NotFound(format!(
                "Milestone with id={} not found",
                milestone_id
            )))
    })
}

// Verify Milestone Function
#[ic_cdk::update]
fn verify_milestone(payload: VerifyMilestonePayload) -> Result<Milestone, Message> {
    MILESTONES.with(|milestones| {
        let mut milestones = milestones.borrow_mut();
        if let Some(mut milestone) = milestones.remove(&payload.milestone_id) {
            milestone.status = "verified".to_string();
            milestones.insert(payload.milestone_id, milestone.clone());
            Ok(milestone)
        } else {
            Err(Message::NotFound(format!(
                "Milestone with id={} not found",
                payload.milestone_id
            )))
        }
    })
}

// Submit Proof Function
#[ic_cdk::update]
fn submit_proof(payload: SubmitProofPayload) -> Result<ProofOfReproduction, Message> {
    // Validate inputs
    if payload.methodology_hash.is_empty() || payload.results_hash.is_empty() {
        return Err(Message::InvalidPayload(
            "Methodology hash and results hash are required".to_string(),
        ));
    }

    // Check if milestone exists
    MILESTONES.with(|milestones| {
        if milestones.borrow().contains_key(&payload.milestone_id) {
            let id = generate_id();
            let proof = ProofOfReproduction {
                id,
                methodology_hash: payload.methodology_hash,
                results_hash: payload.results_hash,
                status: "pending".to_string(),
            };

            PROOFS.with(|proofs| {
                proofs.borrow_mut().insert(id, proof.clone());
            });

            let mut milestones = milestones.borrow_mut();
            if let Some(mut milestone) = milestones.remove(&payload.milestone_id) {
                milestone.proofs.push(id);
                milestones.insert(payload.milestone_id, milestone);
            }

            Ok(proof)
        } else {
            Err(Message::NotFound(format!(
                "Milestone with id={} not found",
                payload.milestone_id
            )))
        }
    })
}

// Get Proof by ID
#[ic_cdk::query]
fn get_proof_by_id(proof_id: u64) -> Result<ProofOfReproduction, Message> {
    PROOFS.with(|proofs| {
        proofs
            .borrow()
            .get(&proof_id)
            .ok_or(Message::NotFound(format!(
                "Proof with id={} not found",
                proof_id
            )))
    })
}

// Export candid interface
ic_cdk::export_candid!();
