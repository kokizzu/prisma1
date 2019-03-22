use crate::*;
use once_cell::sync::OnceCell;
use std::{
    collections::BTreeSet,
    sync::{Arc, Weak},
};

#[derive(Debug)]
pub struct Fields {
    pub all: Vec<Field>,
    id: OnceCell<Weak<ScalarField>>,
    scalar: OnceCell<Vec<Weak<ScalarField>>>,
    relation: OnceCell<Vec<Weak<RelationField>>>,
    model: ModelWeakRef,
    created_at: OnceCell<Option<Arc<ScalarField>>>,
    updated_at: OnceCell<Option<Arc<ScalarField>>>,
}

impl Fields {
    pub fn new(all: Vec<Field>, model: ModelWeakRef) -> Fields {
        Fields {
            all: all,
            id: OnceCell::new(),
            scalar: OnceCell::new(),
            relation: OnceCell::new(),
            created_at: OnceCell::new(),
            updated_at: OnceCell::new(),
            model,
        }
    }

    pub fn id(&self) -> Arc<ScalarField> {
        self.id
            .get_or_init(|| {
                self.all
                    .iter()
                    .fold(None, |acc, field| match field {
                        Field::Scalar(sf) if sf.is_id() => Some(Arc::downgrade(sf)),
                        _ => acc,
                    })
                    .expect("No id field defined!")
            })
            .upgrade()
            .unwrap()
    }

    pub fn created_at(&self) -> &Option<Arc<ScalarField>> {
        self.created_at.get_or_init(|| {
            self.scalar_weak()
                .iter()
                .map(|sf| sf.upgrade().unwrap())
                .find(|sf| sf.is_created_at())
        })
    }

    pub fn updated_at(&self) -> &Option<Arc<ScalarField>> {
        self.updated_at.get_or_init(|| {
            self.scalar_weak()
                .iter()
                .map(|sf| sf.upgrade().unwrap())
                .find(|sf| sf.is_updated_at())
        })
    }

    pub fn scalar(&self) -> Vec<Arc<ScalarField>> {
        self.scalar_weak().iter().map(|f| f.upgrade().unwrap()).collect()
    }

    fn scalar_weak(&self) -> &[Weak<ScalarField>] {
        self.scalar
            .get_or_init(|| self.all.iter().fold(Vec::new(), Self::scalar_filter))
            .as_slice()
    }

    pub fn relation(&self) -> Vec<Arc<RelationField>> {
        self.relation_weak().iter().map(|f| f.upgrade().unwrap()).collect()
    }

    fn relation_weak(&self) -> &[Weak<RelationField>] {
        self.relation
            .get_or_init(|| self.all.iter().fold(Vec::new(), Self::relation_filter))
            .as_slice()
    }

    pub fn find_many_from_all(&self, names: &BTreeSet<String>) -> Vec<&Field> {
        self.all
            .iter()
            .filter(|field| names.contains(field.db_name().as_ref()))
            .collect()
    }

    pub fn find_many_from_scalar(&self, names: &BTreeSet<String>) -> Vec<Arc<ScalarField>> {
        self.scalar_weak()
            .iter()
            .filter(|field| names.contains(field.upgrade().unwrap().db_name()))
            .map(|field| field.upgrade().unwrap())
            .collect()
    }

    pub fn find_many_from_relation(&self, names: &BTreeSet<String>) -> Vec<Arc<RelationField>> {
        self.relation_weak()
            .iter()
            .filter(|field| names.contains(&field.upgrade().unwrap().db_name()))
            .map(|field| field.upgrade().unwrap())
            .collect()
    }

    pub fn find_from_all(&self, name: &str) -> DomainResult<&Field> {
        self.all.iter().find(|field| field.db_name() == name).ok_or_else(|| {
            DomainError::NotFound(Missing::Field {
                name: name.to_string(),
                model: self.model().name.clone(),
            })
        })
    }

    pub fn find_from_scalar(&self, name: &str) -> DomainResult<Arc<ScalarField>> {
        self.scalar_weak()
            .iter()
            .map(|field| field.upgrade().unwrap())
            .find(|field| field.db_name() == name)
            .ok_or_else(|| {
                DomainError::NotFound(Missing::ScalarField {
                    name: name.to_string(),
                    model: self.model().name.clone(),
                })
            })
    }

    fn model(&self) -> ModelRef {
        self.model.upgrade().unwrap()
    }

    pub fn find_from_relation_fields(&self, name: &str) -> DomainResult<Arc<RelationField>> {
        self.relation_weak()
            .iter()
            .map(|field| field.upgrade().unwrap())
            .find(|field| field.name == name)
            .ok_or_else(|| {
                DomainError::NotFound(Missing::RelationField {
                    name: name.to_string(),
                    model: self.model().name.clone(),
                })
            })
    }

    pub fn find_from_relation(&self, name: &str) -> DomainResult<Arc<RelationField>> {
        self.relation_weak()
            .iter()
            .map(|field| field.upgrade().unwrap())
            .find(|field| field.relation().name == name)
            .ok_or_else(|| {
                DomainError::NotFound(Missing::FieldForRelation {
                    relation: name.to_string(),
                    model: self.model().name.clone(),
                })
            })
    }

    fn scalar_filter(mut acc: Vec<Weak<ScalarField>>, field: &Field) -> Vec<Weak<ScalarField>> {
        if let Field::Scalar(scalar_field) = field {
            acc.push(Arc::downgrade(scalar_field));
        };

        acc
    }

    fn relation_filter<'a>(mut acc: Vec<Weak<RelationField>>, field: &'a Field) -> Vec<Weak<RelationField>> {
        if let Field::Relation(relation_field) = field {
            acc.push(Arc::downgrade(relation_field));
        };

        acc
    }
}