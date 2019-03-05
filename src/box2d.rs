// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::UnknownUnit;
use scale::TypedScale;
use num::*;
use rect::TypedRect;
use point::{point2, TypedPoint2D};
use vector::{vec2, TypedVector2D};
use side_offsets::TypedSideOffsets2D;
use size::TypedSize2D;
use approxord::{min, max};

use num_traits::NumCast;
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use core::borrow::Borrow;
use core::cmp::PartialOrd;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::{Add, Div, Mul, Sub};


/// An axis aligned rectangle represented by its minimum and maximum coordinates.
#[repr(C)]
pub struct TypedBox2D<T, U = UnknownUnit> {
    pub min: TypedPoint2D<T, U>,
    pub max: TypedPoint2D<T, U>,
}

/// The default box 2d type with no unit.
pub type Box2D<T> = TypedBox2D<T, UnknownUnit>;

#[cfg(feature = "serde")]
impl<'de, T: Copy + Deserialize<'de>, U> Deserialize<'de> for TypedBox2D<T, U> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (min, max) = try!(Deserialize::deserialize(deserializer));
        Ok(TypedBox2D::new(min, max))
    }
}

#[cfg(feature = "serde")]
impl<T: Serialize, U> Serialize for TypedBox2D<T, U> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (&self.min, &self.max).serialize(serializer)
    }
}

impl<T: Hash, U> Hash for TypedBox2D<T, U> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.min.hash(h);
        self.max.hash(h);
    }
}

impl<T: Copy, U> Copy for TypedBox2D<T, U> {}

impl<T: Copy, U> Clone for TypedBox2D<T, U> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: PartialEq, U> PartialEq<TypedBox2D<T, U>> for TypedBox2D<T, U> {
    fn eq(&self, other: &Self) -> bool {
        self.min.eq(&other.min) && self.max.eq(&other.max)
    }
}

impl<T: Eq, U> Eq for TypedBox2D<T, U> {}

impl<T: fmt::Debug, U> fmt::Debug for TypedBox2D<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TypedBox2D({:?}, {:?})", self.min, self.max)
    }
}

impl<T: fmt::Display, U> fmt::Display for TypedBox2D<T, U> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Box2D({}, {})", self.min, self.max)
    }
}

impl<T, U> TypedBox2D<T, U> {
    /// Constructor.
    pub fn new(min: TypedPoint2D<T, U>, max: TypedPoint2D<T, U>) -> Self {
        TypedBox2D {
            min,
            max,
        }
    }
}

impl<T, U> TypedBox2D<T, U>
where
    T: Copy + Zero + PartialOrd,
{
    /// Creates a Box2D of the given size, at offset zero.
    #[inline]
    pub fn from_size(size: TypedSize2D<T, U>) -> Self {
        let zero = TypedPoint2D::zero();
        let point = size.to_vector().to_point();
        TypedBox2D::from_points(&[zero, point])
    }
}

impl<T, U> TypedBox2D<T, U>
where
    T: Copy + PartialOrd,
{
    /// Returns true if the box has a negative area.
    ///
    /// The common interpretation for a negative box is to consider it empty. It can be obtained
    /// by calculating the intersection of two boxes that do not intersect.
    #[inline]
    pub fn is_negative(&self) -> bool {
        self.max.x < self.min.x || self.max.y < self.min.y
    }

    /// Returns true if the size is zero or negative.
    #[inline]
    pub fn is_empty_or_negative(&self) -> bool {
        self.max.x <= self.min.x || self.max.y <= self.min.y
    }

    /// Returns true if the two boxes intersect.
    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }

    /// Computes the intersection of two boxes.
    ///
    /// The result is a negative box if the boxes do not intersect.
    #[inline]
    pub fn intersection(&self, other: &Self) -> Self {
        TypedBox2D {
            min: point2(
                max(self.min.x, other.min.x),
                max(self.min.y, other.min.y),
            ),
            max: point2(
                min(self.max.x, other.max.x),
                min(self.max.y, other.max.y),
            )
        }
    }

    /// Computes the intersection of two boxes, returning `None` if the boxes do not intersect.
    #[inline]
    pub fn try_intersection(&self, other: &Self) -> Option<Self> {
        let intersection = self.intersection(other);

        if intersection.is_negative() {
            return None;
        }

        Some(intersection)
    }
}

impl<T, U> TypedBox2D<T, U>
where
    T: Copy + Add<T, Output = T>,
{
    /// Returns the same box, translated by a vector.
    #[inline]
    pub fn translate(&self, by: &TypedVector2D<T, U>) -> Self {
        Self::new(self.min + *by, self.max + *by)
    }
}

impl<T, U> TypedBox2D<T, U>
where
    T: Copy + PartialOrd + Zero,
{
    /// Returns true if this box contains the point. Points are considered
    /// in the box if they are on the front, left or top faces, but outside if they
    /// are on the back, right or bottom faces.
    #[inline]
    pub fn contains(&self, other: &TypedPoint2D<T, U>) -> bool {
        self.min.x <= other.x && other.x < self.max.x
            && self.min.y < other.y && other.y <= self.max.y
    }
}

impl<T, U> TypedBox2D<T, U>
where
    T: Copy + PartialOrd + Zero + Sub<T, Output = T>,
{
    /// Returns true if this box contains the interior of the other box. Always
    /// returns true if other is empty, and always returns false if other is
    /// nonempty but this box is empty.
    #[inline]
    pub fn contains_box(&self, other: &Self) -> bool {
        other.is_empty()
            || (self.min.x <= other.min.x && other.max.x <= self.max.x
                && self.min.y <= other.min.y && other.max.y <= self.max.y)
    }
}

impl<T, U> TypedBox2D<T, U>
where
    T: Copy + Sub<T, Output = T>,
{
    #[inline]
    pub fn size(&self)-> TypedSize2D<T, U> {
        (self.max - self.min).to_size()
    }

    #[inline]
    pub fn to_rect(&self) -> TypedRect<T, U> {
        TypedRect {
            origin: self.min,
            size: self.size(),
        }
    }
}

impl<T, U> TypedBox2D<T, U>
where
    T: Copy + PartialEq + Add<T, Output = T> + Sub<T, Output = T>,
{
    /// Inflates the box by the specified sizes on each side respectively.
    #[inline]
    #[cfg_attr(feature = "unstable", must_use)]
    pub fn inflate(&self, width: T, height: T) -> Self {
        TypedBox2D {
            min: point2(self.min.x - width, self.min.y - height),
            max: point2(self.max.x + width, self.max.x + height),
        }
    }
}

impl<T, U> TypedBox2D<T, U>
where
    T: Copy + Zero + PartialOrd + Add<T, Output = T> + Sub<T, Output = T>,
{
    /// Calculate the size and position of an inner box.
    ///
    /// Subtracts the side offsets from all sides. The horizontal, vertical
    /// and applicate offsets must not be larger than the original side length.
    pub fn inner_box(&self, offsets: TypedSideOffsets2D<T, U>) -> Self {
        let b = TypedBox2D {
            min: self.min + vec2(offsets.left, offsets.top),
            max: self.max - vec2(offsets.right, offsets.bottom),
        };

        debug_assert!(b.size().width >= T::zero());
        debug_assert!(b.size().height >= T::zero());

        b
    }

    /// Calculate the b and position of an outer box.
    ///
    /// Add the offsets to all sides. The expanded box is returned.
    pub fn outer_box(&self, offsets: TypedSideOffsets2D<T, U>) -> Self {
        let b = TypedBox2D {
            min: self.min - vec2(offsets.left, offsets.top),
            max: self.max + vec2(offsets.right, offsets.bottom),
        };

        debug_assert!(b.size().width >= T::zero());
        debug_assert!(b.size().height >= T::zero());

        b
    }
}


impl<T, U> TypedBox2D<T, U>
where
    T: Copy + Zero + PartialOrd,
{
    /// Returns the smallest box containing all of the provided points.
    pub fn from_points<I>(points: I) -> Self
    where
        I: IntoIterator,
        I::Item: Borrow<TypedPoint2D<T, U>>,
    {
        let mut points = points.into_iter();

        // Need at least 2 different points for a valid box (ie: volume > 0).
        let (mut min_x, mut min_y) = match points.next() {
            Some(first) => (first.borrow().x, first.borrow().y),
            None => return TypedBox2D::zero(),
        };
        let (mut max_x, mut max_y) = (min_x, min_y);

        {
            let mut assign_min_max = |point: I::Item| {
                let p = point.borrow();
                if p.x < min_x {
                    min_x = p.x
                }
                if p.x > max_x {
                    max_x = p.x
                }
                if p.y < min_y {
                    min_y = p.y
                }
                if p.y > max_y {
                    max_y = p.y
                }
            };

            match points.next() {
                Some(second) => assign_min_max(second),
                None => return TypedBox2D::zero(),
            }

            for point in points {
                assign_min_max(point);
            }
        }

        TypedBox2D {
            min: point2(min_x, min_y),
            max: point2(max_x, max_y),
        }
    }
}

impl<T, U> TypedBox2D<T, U>
where
    T: Copy + One + Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
{
    /// Linearly interpolate between this box and another box.
    ///
    /// `t` is expected to be between zero and one.
    #[inline]
    pub fn lerp(&self, other: Self, t: T) -> Self {
        Self::new(
            self.min.lerp(other.min, t),
            self.max.lerp(other.max, t),
        )
    }
}

impl<T, U> TypedBox2D<T, U>
where
    T: Copy + One + Add<Output = T> + Div<Output = T>,
{
    pub fn center(&self) -> TypedPoint2D<T, U> {
        let two = T::one() + T::one();
        (self.min + self.max.to_vector()) / two
    }
}

impl<T, U> TypedBox2D<T, U>
where
    T: Copy + Clone + PartialOrd + Add<T, Output = T> + Sub<T, Output = T> + Zero,
{
    #[inline]
    pub fn union(&self, other: &Self) -> Self {
        TypedBox2D {
            min: point2(
                min(self.min.x, other.min.x),
                min(self.min.y, other.min.y),
            ),
            max: point2(
                max(self.max.x, other.max.x),
                max(self.max.y, other.max.y),
            ),
        }
    }
}

impl<T, U> TypedBox2D<T, U>
where
    T: Copy,
{
    #[inline]
    pub fn scale<S: Copy>(&self, x: S, y: S) -> Self
    where
        T: Mul<S, Output = T>
    {
        TypedBox2D {
            min: point2(self.min.x * x, self.min.y * y),
            max: point2(self.max.x * x, self.max.y * y),
        }
    }
}

impl<T, U> TypedBox2D<T, U>
where
    T: Copy + Mul<T, Output = T> + Sub<T, Output = T>,
{
    #[inline]
    pub fn area(&self) -> T {
        let size = self.size();
        size.width * size.height
    }
}

impl<T, U> TypedBox2D<T, U>
where
    T: Copy + Zero,
{
    /// Constructor, setting all sides to zero.
    pub fn zero() -> Self {
        TypedBox2D::new(TypedPoint2D::zero(), TypedPoint2D::zero())
    }
}

impl<T, U> TypedBox2D<T, U>
where
    T: PartialEq,
{
    /// Returns true if the size is zero.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.min.x == self.max.x || self.min.y == self.max.y
    }
}

impl<T, U> Mul<T> for TypedBox2D<T, U>
where
    T: Copy + Mul<T, Output = T>,
{
    type Output = Self;
    #[inline]
    fn mul(self, scale: T) -> Self {
        TypedBox2D::new(self.min * scale, self.max * scale)
    }
}

impl<T, U> Div<T> for TypedBox2D<T, U>
where
    T: Copy + Div<T, Output = T>,
{
    type Output = Self;
    #[inline]
    fn div(self, scale: T) -> Self {
        TypedBox2D::new(self.min / scale, self.max / scale)
    }
}

impl<T, U1, U2> Mul<TypedScale<T, U1, U2>> for TypedBox2D<T, U1>
where
    T: Copy + Mul<T, Output = T>,
{
    type Output = TypedBox2D<T, U2>;
    #[inline]
    fn mul(self, scale: TypedScale<T, U1, U2>) -> TypedBox2D<T, U2> {
        TypedBox2D::new(self.min * scale, self.max * scale)
    }
}

impl<T, U1, U2> Div<TypedScale<T, U1, U2>> for TypedBox2D<T, U2>
where
    T: Copy + Div<T, Output = T>,
{
    type Output = TypedBox2D<T, U1>;
    #[inline]
    fn div(self, scale: TypedScale<T, U1, U2>) -> TypedBox2D<T, U1> {
        TypedBox2D::new(self.min / scale, self.max / scale)
    }
}

impl<T, Unit> TypedBox2D<T, Unit>
where
    T: Copy,
{
    /// Drop the units, preserving only the numeric value.
    pub fn to_untyped(&self) -> Box2D<T> {
        TypedBox2D::new(self.min.to_untyped(), self.max.to_untyped())
    }

    /// Tag a unitless value with units.
    pub fn from_untyped(c: &Box2D<T>) -> TypedBox2D<T, Unit> {
        TypedBox2D::new(
            TypedPoint2D::from_untyped(&c.min),
            TypedPoint2D::from_untyped(&c.max),
        )
    }
}

impl<T0, Unit> TypedBox2D<T0, Unit>
where
    T0: NumCast + Copy,
{
    /// Cast from one numeric representation to another, preserving the units.
    ///
    /// When casting from floating point to integer coordinates, the decimals are truncated
    /// as one would expect from a simple cast, but this behavior does not always make sense
    /// geometrically. Consider using round(), round_in or round_out() before casting.
    pub fn cast<T1: NumCast + Copy>(&self) -> TypedBox2D<T1, Unit> {
        TypedBox2D::new(
            self.min.cast(),
            self.max.cast(),
        )
    }

    /// Fallible cast from one numeric representation to another, preserving the units.
    ///
    /// When casting from floating point to integer coordinates, the decimals are truncated
    /// as one would expect from a simple cast, but this behavior does not always make sense
    /// geometrically. Consider using round(), round_in or round_out() before casting.
    pub fn try_cast<T1: NumCast + Copy>(&self) -> Option<TypedBox2D<T1, Unit>> {
        match (self.min.try_cast(), self.max.try_cast()) {
            (Some(a), Some(b)) => Some(TypedBox2D::new(a, b)),
            _ => None,
        }
    }
}

impl<T, U> TypedBox2D<T, U>
where
    T: Round,
{
    /// Return a box with edges rounded to integer coordinates, such that
    /// the returned box has the same set of pixel centers as the original
    /// one.
    /// Values equal to 0.5 round up.
    /// Suitable for most places where integral device coordinates
    /// are needed, but note that any translation should be applied first to
    /// avoid pixel rounding errors.
    /// Note that this is *not* rounding to nearest integer if the values are negative.
    /// They are always rounding as floor(n + 0.5).
    #[cfg_attr(feature = "unstable", must_use)]
    pub fn round(&self) -> Self {
        TypedBox2D::new(self.min.round(), self.max.round())
    }
}

impl<T, U> TypedBox2D<T, U>
where
    T: Floor + Ceil,
{
    /// Return a box with faces/edges rounded to integer coordinates, such that
    /// the original box contains the resulting box.
    #[cfg_attr(feature = "unstable", must_use)]
    pub fn round_in(&self) -> Self {
        let min = self.min.ceil();
        let max = self.max.floor();
        TypedBox2D { min, max }
    }

    /// Return a box with faces/edges rounded to integer coordinates, such that
    /// the original box is contained in the resulting box.
    #[cfg_attr(feature = "unstable", must_use)]
    pub fn round_out(&self) -> Self {
        let min_x = self.min.x.floor();
        let min_y = self.min.y.floor();
        let max_x = self.max.x.ceil();
        let max_y = self.max.y.ceil();
        TypedBox2D {
            min: point2(min_x, min_y),
            max: point2(max_x, max_y),
        }
    }
}

// Convenience functions for common casts
impl<T: NumCast + Copy, Unit> TypedBox2D<T, Unit> {
    /// Cast into an `f32` box.
    pub fn to_f32(&self) -> TypedBox2D<f32, Unit> {
        self.cast()
    }

    /// Cast into an `f64` box.
    pub fn to_f64(&self) -> TypedBox2D<f64, Unit> {
        self.cast()
    }

    /// Cast into an `usize` box, truncating decimals if any.
    ///
    /// When casting from floating point boxes, it is worth considering whether
    /// to `round()`, `round_in()` or `round_out()` before the cast in order to
    /// obtain the desired conversion behavior.
    pub fn to_usize(&self) -> TypedBox2D<usize, Unit> {
        self.cast()
    }

    /// Cast into an `u32` box, truncating decimals if any.
    ///
    /// When casting from floating point boxes, it is worth considering whether
    /// to `round()`, `round_in()` or `round_out()` before the cast in order to
    /// obtain the desired conversion behavior.
    pub fn to_u32(&self) -> TypedBox2D<u32, Unit> {
        self.cast()
    }

    /// Cast into an `i32` box, truncating decimals if any.
    ///
    /// When casting from floating point boxes, it is worth considering whether
    /// to `round()`, `round_in()` or `round_out()` before the cast in order to
    /// obtain the desired conversion behavior.
    pub fn to_i32(&self) -> TypedBox2D<i32, Unit> {
        self.cast()
    }

    /// Cast into an `i64` box, truncating decimals if any.
    ///
    /// When casting from floating point boxes, it is worth considering whether
    /// to `round()`, `round_in()` or `round_out()` before the cast in order to
    /// obtain the desired conversion behavior.
    pub fn to_i64(&self) -> TypedBox2D<i64, Unit> {
        self.cast()
    }
}

impl<T, U> From<TypedSize2D<T, U>> for TypedBox2D<T, U>
where
    T: Copy + Zero + PartialOrd,
{
    fn from(b: TypedSize2D<T, U>) -> Self {
        Self::from_size(b)
    }
}

#[cfg(test)]
mod tests {
    use side_offsets::SideOffsets2D;
    use size::size2;
    use point::Point2D;
    use super::*;

    #[test]
    fn test_size() {
        let b = Box2D::new(point2(-10.0, -10.0), point2(10.0, 10.0));
        assert_eq!(b.size().width, 20.0);
        assert_eq!(b.size().height, 20.0);
    }

    #[test]
    fn test_center() {
        let b = Box2D::new(point2(-10.0, -10.0), point2(10.0, 10.0));
        assert_eq!(b.center(), Point2D::zero());
    }

    #[test]
    fn test_area() {
        let b = Box2D::new(point2(-10.0, -10.0), point2(10.0, 10.0));
        assert_eq!(b.area(), 400.0);
    }

    #[test]
    fn test_from_points() {
        let b = Box2D::from_points(&[point2(50.0, 160.0), point2(100.0, 25.0)]);
        assert_eq!(b.min, point2(50.0, 25.0));
        assert_eq!(b.max, point2(100.0, 160.0));
    }

    #[test]
    fn test_round_in() {
        let b = Box2D::from_points(&[point2(-25.5, -40.4), point2(60.3, 36.5)]).round_in();
        assert_eq!(b.min.x, -25.0);
        assert_eq!(b.min.y, -40.0);
        assert_eq!(b.max.x, 60.0);
        assert_eq!(b.max.y, 36.0);
    }

    #[test]
    fn test_round_out() {
        let b = Box2D::from_points(&[point2(-25.5, -40.4), point2(60.3, 36.5)]).round_out();
        assert_eq!(b.min.x,-26.0);
        assert_eq!(b.min.y, -41.0);
        assert_eq!(b.max.x, 61.0);
        assert_eq!(b.max.y, 37.0);
    }

    #[test]
    fn test_round() {
        let b = Box2D::from_points(&[point2(-25.5, -40.4), point2(60.3, 36.5)]).round();
        assert_eq!(b.min.x,-26.0);
        assert_eq!(b.min.y, -40.0);
        assert_eq!(b.max.x, 60.0);
        assert_eq!(b.max.y, 37.0);
    }

    #[test]
    fn test_from_size() {
        let b = Box2D::from_size(size2(30.0, 40.0));
        assert!(b.min == Point2D::zero());
        assert!(b.size().width == 30.0);
        assert!(b.size().height == 40.0);
    }

    #[test]
    fn test_inner_box() {
        let b = Box2D::from_points(&[point2(50.0, 25.0), point2(100.0, 160.0)]);
        let b = b.inner_box(SideOffsets2D::new(10.0, 20.0, 5.0, 10.0));
        assert_eq!(b.max.x, 80.0);
        assert_eq!(b.max.y, 155.0);
        assert_eq!(b.min.x, 60.0);
        assert_eq!(b.min.y, 35.0);
    }

    #[test]
    fn test_outer_box() {
        let b = Box2D::from_points(&[point2(50.0, 25.0), point2(100.0, 160.0)]);
        let b = b.outer_box(SideOffsets2D::new(10.0, 20.0, 5.0, 10.0));
        assert_eq!(b.max.x, 120.0);
        assert_eq!(b.max.y, 165.0);
        assert_eq!(b.min.x, 40.0);
        assert_eq!(b.min.y, 15.0);
    }

    #[test]
    fn test_translate() {
        let size = size2(15.0, 15.0);
        let mut center = (size / 2.0).to_vector().to_point();
        let b = Box2D::from_size(size);
        assert_eq!(b.center(), center);
        let translation = vec2(10.0, 2.5);
        let b = b.translate(&translation);
        center += translation;
        assert_eq!(b.center(), center);
        assert_eq!(b.max.x, 25.0);
        assert_eq!(b.max.y, 17.5);
        assert_eq!(b.min.x, 10.0);
        assert_eq!(b.min.y, 2.5);
    }

    #[test]
    fn test_union() {
        let b1 = Box2D::from_points(&[point2(-20.0, -20.0), point2(0.0, 20.0)]);
        let b2 = Box2D::from_points(&[point2(0.0, 20.0), point2(20.0, -20.0)]);
        let b = b1.union(&b2);
        assert_eq!(b.max.x, 20.0);
        assert_eq!(b.max.y, 20.0);
        assert_eq!(b.min.x, -20.0);
        assert_eq!(b.min.y, -20.0);
    }

    #[test]
    fn test_intersects() {
        let b1 = Box2D::from_points(&[point2(-15.0, -20.0), point2(10.0, 20.0)]);
        let b2 = Box2D::from_points(&[point2(-10.0, 20.0), point2(15.0, -20.0)]);
        assert!(b1.intersects(&b2));
    }

    #[test]
    fn test_intersection() {
        let b1 = Box2D::from_points(&[point2(-15.0, -20.0), point2(10.0, 20.0)]);
        let b2 = Box2D::from_points(&[point2(-10.0, 20.0), point2(15.0, -20.0)]);
        let b = b1.intersection(&b2);
        assert_eq!(b.max.x, 10.0);
        assert_eq!(b.max.y, 20.0);
        assert_eq!(b.min.x, -10.0);
        assert_eq!(b.min.y, -20.0);
    }

    #[test]
    fn test_try_intersection() {
        let b1 = Box2D::from_points(&[point2(-15.0, -20.0), point2(10.0, 20.0)]);
        let b2 = Box2D::from_points(&[point2(-10.0, 20.0), point2(15.0, -20.0)]);
        assert!(b1.try_intersection(&b2).is_some());

        let b1 = Box2D::from_points(&[point2(-15.0, -20.0), point2(-10.0, 20.0)]);
        let b2 = Box2D::from_points(&[point2(10.0, 20.0), point2(15.0, -20.0)]);
        assert!(b1.try_intersection(&b2).is_none());
    }

    #[test]
    fn test_scale() {
        let b = Box2D::from_points(&[point2(-10.0, -10.0), point2(10.0, 10.0)]);
        let b = b.scale(0.5, 0.5);
        assert_eq!(b.max.x, 5.0);
        assert_eq!(b.max.y, 5.0);
        assert_eq!(b.min.x, -5.0);
        assert_eq!(b.min.y, -5.0);
    }

    #[test]
    fn test_lerp() {
        let b1 = Box2D::from_points(&[point2(-20.0, -20.0), point2(-10.0, -10.0)]);
        let b2 = Box2D::from_points(&[point2(10.0, 10.0), point2(20.0, 20.0)]);
        let b = b1.lerp(b2, 0.5);
        assert_eq!(b.center(), Point2D::zero());
        assert_eq!(b.size().width, 10.0);
        assert_eq!(b.size().height, 10.0);
    }

    #[test]
    fn test_contains() {
        let b = Box2D::from_points(&[point2(-20.0, -20.0), point2(20.0, 20.0)]);
        assert!(b.contains(&point2(-15.3, 10.5)));
    }

    #[test]
    fn test_contains_box() {
        let b1 = Box2D::from_points(&[point2(-20.0, -20.0), point2(20.0, 20.0)]);
        let b2 = Box2D::from_points(&[point2(-14.3, -16.5), point2(6.7, 17.6)]);
        assert!(b1.contains_box(&b2));
    }

    #[test]
    fn test_inflate() {
        let b = Box2D::from_points(&[point2(-20.0, -20.0), point2(20.0, 20.0)]);
        let b = b.inflate(10.0, 5.0);
        assert_eq!(b.size().width, 60.0);
        assert_eq!(b.size().height, 50.0);
        assert_eq!(b.center(), Point2D::zero());
    }

    #[test]
    fn test_is_empty() {
        for i in 0..2 {
            let mut coords_neg = [-20.0, -20.0];
            let mut coords_pos = [20.0, 20.0];
            coords_neg[i] = 0.0;
            coords_pos[i] = 0.0;
            let b = Box2D::from_points(&[Point2D::from(coords_neg), Point2D::from(coords_pos)]);
            assert!(b.is_empty());
        }
    }
}
